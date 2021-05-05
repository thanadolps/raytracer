use itertools::Itertools;
use nalgebra::{Point3, Similarity3, Translation3, Unit, UnitQuaternion, Vector3};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use enum_dispatch::enum_dispatch;

use super::{AREALIGHT_FINITEDIFF_LENGTH, AREALIGHT_MONTECARLO_SAMPLE};
// use super::Color3;
use super::Color3;
use super::renderer::raycast;
use super::Scene;

#[enum_dispatch]
pub trait Light {
	// intensity of light at position=pos at normal=norm factored in normal attenuation
	fn direct_light_at(&self, pos: Point3<f32>, norm: Unit<Vector3<f32>>, scene: &Scene) -> Color3;
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
	light: Color3
}

impl PointLight {
	pub fn new(pos: Point3<f32>, light: Color3) -> Self {
		PointLight {pos, light}
	}
	
	fn _light_at(
		self_pos: Point3<f32>,
		self_light: Color3,
		pos: Point3<f32>, 
		norm: Unit<Vector3<f32>>,
		scene: &Scene)
		-> Color3 {
			
		let (dir_to_obj, dist_to_obj) = Unit::new_and_get(pos - self_pos);
		let norm_attune = -norm.dot(dir_to_obj.as_ref());
		
		if norm_attune <= 0.0 {
			return [0.0, 0.0, 0.0].into();
		}
		
		let hit_info = raycast(scene, self_pos, dir_to_obj);
		 
		match hit_info {
			None => scene.get_skylight(),
			Some(hit) => {
				// check if it hit something before reaching objct?
				// 1e-4 is for mitigate float unstable comparision
				if hit.dist + 1e-4 < dist_to_obj{
					// hit something before the object (or at least near enough)
					Color3::from([0.0, 0.0, 0.0])
				}
				else {
					// actually hit object
					(self_light / (dist_to_obj * dist_to_obj)) * norm_attune
				}
			},
		}
	
	}
}

impl Light for PointLight {
	fn direct_light_at(&self, pos: Point3<f32>, norm: Unit<Vector3<f32>>, scene: &Scene) -> Color3 {
		Self::_light_at(self.pos, self.light, pos, norm, scene)	
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
		DirectionalLight {dir, light}
	}
}

impl Light for DirectionalLight {
	fn direct_light_at(&self, pos: Point3<f32>, norm: Unit<Vector3<f32>>, scene: &Scene) -> Color3 {
		
		let norm_attune = -norm.dot(self.dir.as_ref());
		
		if norm_attune <= 0.0 {
			return scene.get_skylight();
		}
		
		let hit_info = raycast(scene, pos, -self.dir);
		
		match hit_info {
			None => self.light * norm_attune,
			Some(_) => scene.get_skylight(),
		}
	}
}


// Area Light
#[derive(Serialize, Deserialize)]
pub struct AreaLight {
	// #[serde(flatten)]
	transformer: Similarity3<f32>,
	light: Color3,
}

impl AreaLight{
	pub fn new(center: Vector3<f32>, rotation: UnitQuaternion<f32>, scaling: f32, light: Option<Color3>) -> Self {
		AreaLight {
			transformer: Similarity3::from_parts(
				Translation3::from(center), rotation, scaling
			),
			light: light.unwrap_or_else(|| Color3::new(1.0, 1.0, 1.0))
		}
	}
	
	fn _light_at_monte_carol(
		self_trans: Similarity3<f32>,
		self_light: Color3,
		pos: Point3<f32>,
		norm: Unit<Vector3<f32>>,
		scene: &Scene) -> Color3 {
		
		let mut rng = thread_rng();
		// TODO: Move to global or struct?
		let distribution = Uniform::new_inclusive(-1.0, 1.0);
		
		(0..AREALIGHT_MONTECARLO_SAMPLE).map( |_| {
			
			let transformed_point = self_trans * 
					Point3::new(0.0, distribution.sample(&mut rng), distribution.sample(&mut rng));
			
			PointLight::_light_at(transformed_point, self_light, pos, norm, scene)
		}).sum::<Color3>() / (AREALIGHT_MONTECARLO_SAMPLE as f32)
	}
	
	fn _light_at_finite_diff(
	self_trans: Similarity3<f32>,
	self_light: Color3,
	pos: Point3<f32>,
	norm: Unit<Vector3<f32>>,
	scene: &Scene) -> Color3 {
		
		const SQRT_RAY_COUNT: u32 = 2 * AREALIGHT_FINITEDIFF_LENGTH + 1;
		const FD_LENGTH: i32 = AREALIGHT_FINITEDIFF_LENGTH as i32;
		const FD_FLOAT: f32 = FD_LENGTH as f32;
		
		(-FD_LENGTH..=FD_LENGTH)
				.map(|x| (x as f32)/FD_FLOAT)
				.cartesian_product((-FD_LENGTH..=FD_LENGTH).map(|x| (x as f32)/FD_FLOAT))
				.map( |(i, j)| {
					let transformed_point = self_trans * Point3::new(0.0, i, j);
					PointLight::_light_at(transformed_point, self_light, pos, norm, scene)
				}).sum::<Color3>() / (SQRT_RAY_COUNT * SQRT_RAY_COUNT) as f32
	}
}

impl Light for AreaLight {
	fn direct_light_at(&self, pos: Point3<f32>, norm: Unit<Vector3<f32>>, scene: &Scene) -> Color3 {
		Self::_light_at_finite_diff(self.transformer, self.light, pos, norm, scene)
	}
}