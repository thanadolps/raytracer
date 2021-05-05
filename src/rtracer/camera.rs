use nalgebra::{Point3, Rotation3, Unit, Vector3};
use serde::{Deserialize, Serialize};
use serde;



#[derive(Serialize, Deserialize, Clone)]
// #[serde(into = "CameraSerdeInterface")]
// #[serde(from = "CameraSerdeInterface")]
pub struct Camera {
	pub pos: Point3<f32>,
	forward: Vector3<f32>,
	right: Vector3<f32>,
	up: Vector3<f32>,
}

impl Camera {
	pub fn new(pos: Point3<f32>, rot: Rotation3<f32>) -> Camera {
		Camera {
			pos,
			forward: rot * Vector3::new(1.0, 0.0, 0.0),
			right: rot * Vector3::new(0.0, 1.0, 0.0),
			up: rot * Vector3::new(0.0, 0.0, 1.0),
		}
	}
	pub fn ray_at_pixel_position(&self, px:u32, py:u32, unit_per_pixel: f32, half_width: u32, half_height: u32)
		-> Unit<Vector3<f32>> {
		let i = unit_per_pixel * (px as i32 - half_width as i32) as f32;
		let j = unit_per_pixel * (py as i32 - half_height as i32) as f32;
		
		let dir = self.forward + i * self.right - j * self.up;
		
		Unit::new_normalize(dir)
	}
	pub fn get_rotation(&self) -> Rotation3<f32> {
		Rotation3::face_towards(&self.forward, &self.up)
	}
}
