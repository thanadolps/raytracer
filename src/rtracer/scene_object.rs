use serde::{Deserialize, Serialize};

use super::shape::geometric::Shapes;
use super::Materials;

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneObject {
    pub material: Materials,
    pub shape: Shapes,
}

impl SceneObject {
    pub fn new(shape: impl Into<Shapes>, material: impl Into<Materials>) -> Self {
        SceneObject {
            material: material.into(),
            shape: shape.into(),
        }
    }
}
