
use std::slice::Iter;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

use super::{Color3, light, SceneObject};

#[derive(Serialize, Deserialize)]
pub struct Scene {
	objects: Vec<SceneObject>,
	lights: Vec<light::Lights>,
	skylight: Color3,
}

impl Scene {

	pub fn new() -> Scene {
		Scene {
			objects: Vec::new(),
			lights: Vec::new(),
			skylight: Color3::new(0.0, 0.0, 0.0),
		}
	}

	pub fn from_maybe_component(
		objs: Option<Vec<SceneObject>>,
		lights: Option<Vec<light::Lights>>,
		skylight: Option<Color3>
	) -> Scene {
		Scene {
			objects: objs.unwrap_or_else(Vec::new),
			lights: lights.unwrap_or_else(Vec::new),
			skylight: skylight.unwrap_or_else(|| Color3::new(0.0, 0.0, 0.0)),
		}
	}
	

	pub fn add_obj(&mut self, obj: impl Into<SceneObject>) {
		self.objects.push(obj.into());
	}
	
	pub fn add_light(&mut self, light: light::Lights) {
		self.lights.push(light);
	}

	pub fn append_objs(&mut self, mut objs: Vec<SceneObject>) {
		self.objects.append(&mut objs);
	}

	pub fn append_light(&mut self, mut lights: Vec<light::Lights>) {
		self.lights.append(&mut lights);
	}

	
	pub fn iter_obj(&self) -> Iter<SceneObject> {
		self.objects.iter()
	}
	
	pub fn iter_light(&self) -> Iter<light::Lights> {
		self.lights.iter()
	}
	
	pub fn get_skylight(&self) -> Color3 {
		self.skylight
	}
	
}