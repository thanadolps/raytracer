use nalgebra::{Unit, UnitQuaternion, Vector3};
use rand::Rng;

use rand_distr::{Distribution, UnitSphere};
use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use crate::rtracer::renderer::raycast_compute_light;
use crate::rtracer::thread_buffer::ThreadBuffer;
use crate::rtracer::{
    helper, light::Light, Color3, HitInfo, RayCastInfo, Scene, SceneObject, INDIRECT_DEPTH_LIMIT,
    REFLECTION_DEPTH_LIMIT,
};
use num_traits::One;
use std::f32::consts::PI;

#[enum_dispatch]
pub trait Material {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        hit_object: &SceneObject,
        raycase_info: RayCastInfo,
    ) -> Color3;
}

#[enum_dispatch(Material)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Materials {
    NormalDebug,
    Diffuse,
    PBRDiffuse,
    Reflective,
    PerfectReflective,
    Emission, // PBRReflective
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalDebug {
    #[serde(default = "f32::one")]
    scaler: f32,
}

impl Material for NormalDebug {
    fn compute_light(
        &self,
        _scene: &Scene,
        _thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        _raycase_info: RayCastInfo,
    ) -> Color3 {
        hit_info
            .normal
            .into_owned()
            .map(|v| self.scaler * (v + 1.0) / 2.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Diffuse {
    color: Color3,
    albedo: f32,
}

impl Diffuse {
    pub fn new(color: Color3, albedo: f32) -> Self {
        Diffuse { color, albedo }
    }
}

impl Material for Diffuse {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        _: RayCastInfo,
    ) -> Color3 {
        let dl = scene.direct_light_at(hit_info.intersection, hit_info.normal, thread_buffer);

        let combined_light = self.albedo * scene.get_skylight() + (1.0 - self.albedo) * dl;

        combined_light.component_mul(&self.color) // factor in material's color
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(from="crate::utils::proxy_serialize::PBRDiffuseProxy")]
pub struct PBRDiffuse {
    // albedo * color
    pub color_albedo: Color3,
    pub iteration: usize,
}

impl Material for PBRDiffuse {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        raycast_info: RayCastInfo,
    ) -> Color3 {
        let direct_light =
            scene.direct_light_at(hit_info.intersection, hit_info.normal, thread_buffer)
                / std::f32::consts::PI;

        let total_light = if raycast_info.ray_depth() <= INDIRECT_DEPTH_LIMIT {
            let indirect_light = (0..self.iteration)
                .map(|_| {
                    let reflect_dir = {
                        // this generate random vector with pdf ~ cos(theta)
                        // notice that it generate a vector from surface of sphere with center on the tip of normal vector
                        // It's the fact that a probability of getting a vector that lie sphere surface cross section is the same
                        // for all cross section
                        // so a pdf of vector is proportional to the length of the cross section, which is a circle with radius cos(theta)
                        let offset = Vector3::from(UnitSphere.sample(&mut thread_buffer.rng));
                        let random_vector = hit_info.normal.into_owned() + offset;
                        Unit::new_normalize(random_vector)
                    };

                    // there's no need for weakening factor (eg. cos(theta))
                    // since it's already accounted for in the sampling.
                    // that is, the pdf ~ cos(theta)
                    raycast_compute_light(
                        scene,
                        thread_buffer,
                        hit_info.intersection,
                        reflect_dir,
                        raycast_info,
                    )
                })
                .sum::<Color3>()
                / (self.iteration as f32);

            direct_light + indirect_light
        } else {
            direct_light
        };

        total_light.component_mul(&self.color_albedo)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reflective {
    color: Color3,
    roughness: f32,
    iteration: usize,
}

impl Reflective {
    pub fn new(color: Color3, roughness: f32, iteration: usize) -> Self {
        Reflective {
            color,
            roughness,
            iteration,
        }
    }

    /*fn gen_pos_noise(&self, noise_fn: impl NoiseFn<[f64; 3]>, pos: Vector3<f32>, rng: &mut impl Rng)
        -> f32 {
        let ball_random: Vector3<f32> = Vector3::from(UnitBall.sample(rng)) * self.noise_variance;
        let n: [f64; 3] = ((pos + ball_random) / self.smoothness).map(f64::from).into();
        noise_fn.get(n) as f32
    }

    fn _compute_light_perlin(&self, scene: &Scene, hit_info: &HitInfo,
                     hit_object: &SceneObject, rng: &mut impl Rng) -> Color3 {
        use crate::rtracer::renderer::raycast_compute_light;
        use rand_distr::UnitBall;

        let noise_gen = noise::Perlin::new();

        // let mut sum = nalgebra::zero::<Vector3<f32>>();
        (0..self.iteration).map(|_| {
            let mut gpn =
                || self.gen_pos_noise(noise_gen, hit_info.intersection.coords, rng);

            let (a, b, c) = (gpn(), gpn(), gpn());
            let reflection_noise = self.noise_multiplier * Vector3::new(a, b, c);
            let reflect_dir =
                Unit::new_normalize(
                    helper::calculate_reflect_ray(
                        &hit_info.incoming_dir,
                        &hit_info.normal).into_inner()
                        + reflection_noise
                );
            raycast_compute_light(
                scene,
                hit_info.intersection.clone(),
                reflect_dir,
                rng
            )
        }).sum::<Color3>() / (self.iteration as f32)
    }*/

    fn _compute_light_unbiased(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        raycast_info: RayCastInfo,
    ) -> Color3 {
        use rand_distr::UnitBall;

        let r = self.roughness;
        let r_sq = r * r;
        let pdf_denominator = (4.0 * std::f32::consts::PI) * (r * r - (2.0 / 3.0));

        let mean_light = (0..self.iteration)
            .map(|_| {
                let (reflect_dir, weakening_factor, cos_angle) = loop {
                    let reflect_noise = r * Vector3::from(UnitBall.sample(&mut thread_buffer.rng));
                    let perfect_reflection =
                        helper::calculate_reflect_ray(&hit_info.incoming_dir, &hit_info.normal);
                    let reflect_dir =
                        Unit::new_normalize(perfect_reflection.into_inner() + reflect_noise);

                    let weakening_factor = hit_info.normal.dot(&reflect_dir);
                    if weakening_factor.is_sign_positive() {
                        let cos_angle = perfect_reflection.dot(&reflect_dir);
                        break (reflect_dir, weakening_factor, cos_angle);
                    }
                };

                let pdf_numerator = {
                    let sin_sq = 1.0 - cos_angle * cos_angle;
                    (r_sq - sin_sq).sqrt()
                };

                raycast_compute_light(
                    scene,
                    thread_buffer,
                    hit_info.intersection.clone(),
                    reflect_dir,
                    raycast_info,
                ) * (weakening_factor / pdf_numerator)
            })
            .sum::<Color3>()
            * pdf_denominator
            / (self.iteration as f32);

        mean_light.component_mul(&self.color)
    }

    // TODO: this can be potentially faster than monte carlo, but I can't figure out implementation as of now
    // need a way to generate finite ray in unit sphere uniformly (UnitBall but deterministic)
    // reading implementation of UnitBall might be good
    fn _compute_light_finite(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        raycase_info: RayCastInfo,
    ) -> Color3 {
        // iterator that yield RES^2 uniform unit vector on sphere surface

        let qx: UnitQuaternion<f32> = UnitQuaternion::from_axis_angle(
            &Vector3::x_axis(),
            2.0 * std::f32::consts::PI / self.iteration as f32,
        );
        let qz: UnitQuaternion<f32> = UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            2.0 * std::f32::consts::PI / self.iteration as f32,
        );

        let reflect_noises = (0..self.iteration)
            .scan(Vector3::y_axis(), |uy: &mut Unit<Vector3<f32>>, _| {
                *uy = qx * *uy; // rotate unit vector on x axis every iteration
                Some(*uy) // yield the rotated vector
            }) // iterator yield unit vector rotating around x-axis, end when completed 1 revolution
            .flat_map(|v| {
                (0..self.iteration).scan(v, |uxy, _| {
                    *uxy = qz * *uxy; // rotate unit vector on z axis every iteration
                    Some(*uxy) // yield that rotated vector
                })
            }) // iterator yield unit vector on sphere surface
            .flat_map(|v| {
                // FIXME: this is still on sphere surface rather than ball volume
                (1..=self.iteration).map(move |x| {
                    let r = x as f32 / self.iteration as f32;
                    v.into_inner().scale(r * self.roughness)
                })
            });

        let mean_light = reflect_noises
            .map(|reflect_noise: Vector3<f32>| {
                let reflect_dir = Unit::new_normalize(
                    helper::calculate_reflect_ray(&hit_info.incoming_dir, &hit_info.normal)
                        .into_inner()
                        + reflect_noise,
                );
                raycast_compute_light(
                    scene,
                    thread_buffer,
                    hit_info.intersection.clone(),
                    reflect_dir,
                    raycase_info,
                )
            })
            .sum::<Color3>()
            / (self.iteration * self.iteration * self.iteration) as f32;

        mean_light.component_mul(&self.color)
    }
}

impl Material for Reflective {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        hit_object: &SceneObject,
        raycast_info: RayCastInfo,
    ) -> Color3 {
        let direct_light =
            scene.direct_light_at(hit_info.intersection, hit_info.normal, thread_buffer);

        if raycast_info.ray_depth() > REFLECTION_DEPTH_LIMIT {
            return direct_light;
        }

        let indirect_light =
            self._compute_light_unbiased(scene, thread_buffer, hit_info, hit_object, raycast_info);

        0.5 * (direct_light + indirect_light)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerfectReflective {
    color: Color3,
}

impl PerfectReflective {
    pub fn new(color: Color3) -> Self {
        PerfectReflective { color }
    }
}

impl Material for PerfectReflective {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        _hit_object: &SceneObject,
        raycast_info: RayCastInfo,
    ) -> Color3 {
        if raycast_info.ray_depth() > REFLECTION_DEPTH_LIMIT {
            return scene
                .direct_light_at(hit_info.intersection, hit_info.normal, thread_buffer)
                .component_mul(&self.color);
        }

        let reflect_dir = helper::calculate_reflect_ray(&hit_info.incoming_dir, &hit_info.normal);

        let reflection_light = raycast_compute_light(
            scene,
            thread_buffer,
            hit_info.intersection.clone(),
            reflect_dir,
            raycast_info,
        );

        reflection_light.component_mul(&self.color)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PBRReflective {
    scope: f32,
    color: Color3,
    albedo: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Emission {
    light: Color3,
}

impl Material for Emission {
    fn compute_light(
        &self,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
        hit_info: &HitInfo,
        hit_object: &SceneObject,
        raycase_info: RayCastInfo,
    ) -> Color3 {
        self.light
    }
}
