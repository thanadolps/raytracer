use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use nalgebra::{Matrix, Point3, U1, U3, Vector3};
use nalgebra::base::allocator::Allocator;
use nalgebra::base::default_allocator::DefaultAllocator;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize, Serializer};

use custom_error::custom_error;

use crate::rtracer::{Color3, light, SceneObject};

use super::{Camera, Scene};

#[derive(Serialize, Deserialize)]
pub struct SceneData {
	pub scene: Scene,
	pub camera: Camera,
}

custom_error!{ pub SceneParserError
	RonSerdeError {source: ron::error::Error} = "Encounter error while (de)serializing scene data",
	IOError {source: io::Error} = "Encounter error while opening file"
}

pub fn load_scene_data(path: impl AsRef<Path>) -> Result<SceneData, SceneParserError> {
	let scene_data = ron::de::from_reader(fs::File::open(path)?)?;
	Ok(scene_data)
}

pub fn save_scene_data(path: impl AsRef<Path>, scene: &SceneData) -> Result<(), SceneParserError> {
	let encoded_scene = to_string_pretty(scene, PrettyConfig::default())?;
	fs::write(path, encoded_scene)?;
	Ok(())
}

pub mod serde_interface {
	use nalgebra::{Point3, Rotation3, Vector3};
	use serde::{Deserialize, Serialize};

	use super::super::camera::Camera;

	#[derive(Serialize, Deserialize)]
	pub struct CameraSerdeInterface {
		position: Point3<f32>,
		rotation: (f32, f32, f32),
	}

	impl From<CameraSerdeInterface> for Camera {
		fn from(inter: CameraSerdeInterface) -> Camera {
			Camera::new(
				inter.position,
				Rotation3::from_euler_angles(inter.rotation.0, inter.rotation.1, inter.rotation.2)
			)
		}
	}

	impl From<Camera> for CameraSerdeInterface {
		fn from(camera: Camera) -> CameraSerdeInterface {
			CameraSerdeInterface {
				position: camera.pos,
				rotation: camera.get_rotation().euler_angles(),
			}
		}
	}

}