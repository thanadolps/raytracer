use itertools::Itertools;
use nalgebra::{Point3, Similarity3, Translation3, Unit, UnitQuaternion, Vector3};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use super::{AREALIGHT_FINITEDIFF_LENGTH, AREALIGHT_MONTECARLO_SAMPLE};
// use super::Color3;
use super::renderer::raycast;
use super::Color3;
use super::Scene;
use crate::rtracer::geometric::Plane;
use crate::rtracer::thread_buffer::ThreadBuffer;

#[enum_dispatch]
pub trait Light {
    // intensity of light at position=pos at normal=norm factored in normal attenuation
    fn direct_light_at(
        &self,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3;
}

#[enum_dispatch(Light)]
#[derive(Serialize, Deserialize)]
pub enum Lights {
    PointLight,
    DirectionalLight,
    AreaLight,
}

// Point Light
#[derive(Serialize, Deserialize)]
pub struct PointLight {
    pos: Point3<f32>,
    light: Color3,
}

impl PointLight {
    pub fn new(pos: Point3<f32>, light: Color3) -> Self {
        PointLight { pos, light }
    }

    // calculated reduction of light due to weakening factor (cos(theta) term) and distance(1/r^2 term)
    fn _calc_reduction_factor(
        self_pos: Point3<f32>,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> f32 {
        let (dir_to_obj, dist_to_obj) = Unit::new_and_get(pos - self_pos);
        let norm_attune = -norm.dot(dir_to_obj.as_ref());

        if norm_attune.is_sign_negative() {
            return 0.0;
        }
        let unobstructed_factor = || norm_attune / (dist_to_obj * dist_to_obj); // lazy eval

        let hit_info = raycast(scene, self_pos, dir_to_obj, &mut thread_buffer.bvh_buffer);

        match hit_info {
            None => unobstructed_factor(),
            Some(hit) => {
                // check if it hit something before reaching objct?
                // 1e-4 is for mitigate float unstable comparison
                if hit.dist + 1e-4 < dist_to_obj {
                    // hit something before the object (or at least near enough)
                    0.0
                } else {
                    // actually hit object
                    unobstructed_factor()
                }
            }
        }
    }

    fn _light_at(
        self_pos: Point3<f32>,
        self_light: Color3,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3 {
        let a = Self::_calc_reduction_factor(self_pos, pos, norm, scene, thread_buffer);
        //if a != 0.0 {*/
            self_light * a
        /*}
        else {
            Color3::zeros()
        }*/
    }
}

impl Light for PointLight {
    fn direct_light_at(
        &self,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3 {
        Self::_light_at(self.pos, self.light, pos, norm, scene, thread_buffer)
    }
}

// Direction Light
#[derive(Serialize, Deserialize)]
pub struct DirectionalLight {
    dir: Unit<Vector3<f32>>,
    light: Color3,
}

impl DirectionalLight {
    pub fn new(dir: Unit<Vector3<f32>>, light: Color3) -> DirectionalLight {
        DirectionalLight { dir, light }
    }
}

impl Light for DirectionalLight {
    fn direct_light_at(
        &self,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3 {
        let norm_attune = -norm.dot(self.dir.as_ref());

        if norm_attune <= 0.0 {
            return Color3::zeros();
        }

        let hit_info = raycast(scene, pos, -self.dir, &mut thread_buffer.bvh_buffer);

        match hit_info {
            None => self.light * norm_attune,
            Some(_) => Color3::zeros(),
        }
    }
}

// Area Light
#[derive(Serialize, Deserialize)]
#[serde(from = "crate::utils::proxy_serialize::AreaLightProxy")]
pub struct AreaLight {
    // #[serde(flatten)]
    pub(crate) plane: Plane,
    pub(crate) light: Color3,
}

impl AreaLight {
    /*
    pub fn new(
        center: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scaling: f32,
        light: Option<Color3>,
    ) -> Self {
        AreaLight {
            transformer: Similarity3::from_parts(Translation3::from(center), rotation, scaling),
            light: light.unwrap_or_else(|| Color3::new(1.0, 1.0, 1.0)),
        }
    }

    fn _light_at_monte_carol(
        self_trans: Similarity3<f32>,
        self_light: Color3,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer
    ) -> Color3 {
        let mut rng = thread_rng();
        // TODO: Move to global or struct?
        let distribution = Uniform::new_inclusive(-1.0, 1.0);

        (0..AREALIGHT_MONTECARLO_SAMPLE)
            .map(|_| {
                let transformed_point = self_trans
                    * Point3::new(
                        0.0,
                        distribution.sample(&mut rng),
                        distribution.sample(&mut rng),
                    );

                PointLight::_light_at(transformed_point, self_light, pos, norm, scene, thread_buffer)
            })
            .sum::<Color3>()
            / (AREALIGHT_MONTECARLO_SAMPLE as f32)
    }*/
}

impl Light for AreaLight {
    #[inline]
    fn direct_light_at(
        &self,
        pos: Point3<f32>,
        norm: Unit<Vector3<f32>>,
        scene: &Scene,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3 {
        const SQRT_RAY_COUNT: u32 = 2 * AREALIGHT_FINITEDIFF_LENGTH + 1;
        const RAY_COUNT: f32 = (SQRT_RAY_COUNT * SQRT_RAY_COUNT) as f32;
        const FD_LENGTH: i32 = AREALIGHT_FINITEDIFF_LENGTH as i32;
        const FD_FLOAT: f32 = FD_LENGTH as f32;

        let Plane {
            pos: plane_pos,
            span_dir,
            span_length,
            cospan_dir,
            cospan_length,
            ..
        } = &self.plane;

        let span = span_dir.scale(*span_length);
        let cospan = cospan_dir.scale(*cospan_length);

        let d_span = span_dir.scale(span_length / FD_FLOAT);
        let d_cospan = cospan_dir.scale(cospan_length / FD_FLOAT);

        // dbg!(d_cospan);
        let mut reduction_factor_sum = 0.0;

        let mut light_point = (plane_pos - (span + cospan));
        for _ in 0..SQRT_RAY_COUNT {
            let mut light_point_i = light_point.clone();
            for _ in 0..SQRT_RAY_COUNT {
                // simulate area light as a lot of point light
                // dbg!(light_point_i);
                reduction_factor_sum += PointLight::_calc_reduction_factor(light_point_i, pos, norm, scene, thread_buffer);
                light_point_i += &d_cospan;
            }
            light_point += &d_span
        }


         self.light * reduction_factor_sum / RAY_COUNT
    }
}
