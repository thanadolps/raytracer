
use nalgebra::{Unit, Vector3, UnitQuaternion};
use rand::{Rng};

use rand_distr::{Distribution};
use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use crate::rtracer::{Color3, helper, HitInfo, light::Light, RayCastInfo, REFLECTION_DEPTH_LIMIT, Scene, SceneObject};
use crate::rtracer::renderer::raycast_compute_light;

#[enum_dispatch]
pub trait Material {
    fn compute_light(
        &self,
        scene: &Scene,
        hit_info: &HitInfo,
        hit_object: &SceneObject,
        raycase_info: RayCastInfo,
        rng: &mut impl Rng)
        -> Color3;
}

#[enum_dispatch(Material)]
#[derive(Serialize, Deserialize)]
pub enum Materials {
    Diffuse,
    Reflective,
    PerfectReflective,
}

#[derive(Serialize, Deserialize)]
pub struct Diffuse {
    color: Color3,
}

impl Diffuse {
    pub fn new(color: Color3) -> Self {
        Diffuse {color}
    }
}

impl Material for Diffuse {
    fn compute_light(&self, scene: &Scene, hit_info: &HitInfo,
                     _hit_object: &SceneObject, _: RayCastInfo, _rng: &mut impl Rng)
        -> Color3 {
        scene.iter_light()
            .map(|x| x.direct_light_at(hit_info.intersection, hit_info.normal, scene))
            .sum::<Color3>()
            .component_mul(&self.color)  // factor in material's color
    }
}


#[derive(Serialize, Deserialize)]
pub struct Reflective {
    roughness: f32,
    iteration: usize,
}

impl Reflective {
    pub fn new(roughness: f32, iteration: usize) -> Self {
        Reflective { roughness, iteration}
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

    fn _compute_light_unbiased(&self, scene: &Scene, hit_info: &HitInfo,
                               _hit_object: &SceneObject, raycast_info: RayCastInfo,
                               rng: &mut impl Rng) -> Color3
    {
        use rand_distr::UnitBall;

        (0..self.iteration)
            .map(|_| {
                let reflect_noise =
                    self.roughness * Vector3::from(UnitBall.sample(rng));
                let reflect_dir =
                    Unit::new_normalize(
                    helper::calculate_reflect_ray(
                        &hit_info.incoming_dir,
                        &hit_info.normal).into_inner()
                        + reflect_noise
                    );
                raycast_compute_light(
                    scene,
                    hit_info.intersection.clone(),
                    reflect_dir,
                    raycast_info,
                    rng
                )
            })
            .sum::<Color3>() / self.iteration as f32
    }

    // TODO: this can be potentially faster than monte carlo, but I can't figure out implementation as of now
    // need a way to generate finite ray in unit sphere uniformly (UnitBall but deterministic)
    // reading implementation of UnitBall might be good
    fn _compute_light_finite(&self, scene: &Scene, hit_info: &HitInfo,
                               _hit_object: &SceneObject, raycase_info: RayCastInfo,
                             rng: &mut impl Rng) -> Color3
    {

        // iterator that yield RES^2 uniform unit vector on sphere surface

        let qx: UnitQuaternion<f32> =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 2.0 * std::f32::consts::PI/self.iteration as f32);
        let qz: UnitQuaternion<f32> =
            UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 2.0 * std::f32::consts::PI/self.iteration as f32);

        let reflect_noises =
            (0..self.iteration)
                .scan(Vector3::y_axis(), |uy: &mut Unit<Vector3<f32>>, _| {
                    *uy = qx * *uy; // rotate unit vector on x axis every iteration
                    Some(*uy)  // yield the rotated vector
                }) // iterator yield unit vector rotating around x-axis, end when completed 1 revolution
                .flat_map(|v|
                    (0..self.iteration).scan(v, |uxy, _| {
                        *uxy = qz * *uxy; // rotate unit vector on z axis every iteration
                        Some(*uxy)  // yield that rotated vector
                    })
                ) // iterator yield unit vector on sphere surface
                .flat_map(|v| {
                    // FIXME: this is still on sphere surface rather than ball volume
                    (1..=self.iteration)
                        .map(move |x| {
                            let r = x as f32/self.iteration as f32;
                            v.into_inner().scale(r * self.roughness)
                        })
                });

        reflect_noises
            .map(|reflect_noise: Vector3<f32>| {
                let reflect_dir =
                    Unit::new_normalize(
                        helper::calculate_reflect_ray(
                            &hit_info.incoming_dir,
                            &hit_info.normal).into_inner()
                            + reflect_noise
                    );
                raycast_compute_light(
                    scene,
                    hit_info.intersection.clone(),
                    reflect_dir,
                    raycase_info,
                    rng
                )
            })
            .sum::<Color3>() / (self.iteration * self.iteration * self.iteration) as f32
    }
}

impl Material for Reflective {
    fn compute_light(&self, scene: &Scene, hit_info: &HitInfo,
                     hit_object: &SceneObject, raycast_info: RayCastInfo, rng: &mut impl Rng)
        -> Color3 {
        if raycast_info.ray_depth() > REFLECTION_DEPTH_LIMIT {
            return scene.get_skylight();
        }
        else {
            self._compute_light_unbiased(scene, hit_info, hit_object, raycast_info, rng)
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct PerfectReflective {
    color: Color3,
}

impl PerfectReflective {
    pub fn new(color: Color3) -> Self {
        PerfectReflective {color}
    }
}

impl Material for PerfectReflective {
    fn compute_light(&self, scene: &Scene, hit_info: &HitInfo,
                     _hit_object: &SceneObject, raycast_info: RayCastInfo, rng: &mut impl Rng)
        -> Color3 {


        if raycast_info.ray_depth() > REFLECTION_DEPTH_LIMIT {
            return scene.get_skylight();
        }

        let reflect_dir =
            helper::calculate_reflect_ray(
                &hit_info.incoming_dir,
                &hit_info.normal
            );

        let reflection_light =
                raycast_compute_light(
                scene,
                hit_info.intersection.clone(),
                reflect_dir,
                raycast_info,
                rng
            );

         reflection_light.component_mul(&self.color)
    }
}