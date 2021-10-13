use crate::rtracer::light::Light;

use std::vec::Vec;

use serde::{Deserialize, Serialize};

use super::{light, Color3, SceneObject};
use nalgebra::{Point3, Unit, Vector3};

#[derive(Serialize, Deserialize)]
pub struct SceneBuilder {
    pub objects: Vec<SceneObject>,
    pub lights: Vec<light::Lights>,
    pub skylight: Color3,
}

impl SceneBuilder {
    pub fn build(self) -> Scene {
        let (bvh, bounded_objects, unbounded_objects) =
            BVHTree::from_scene_objects::<Vec<SceneObject>, _>(self.objects);

        Scene {
            bvh,
            bounded_objects: bounded_objects.into_boxed_slice(),
            unbounded_objects: unbounded_objects.into_boxed_slice(),
            lights: self.lights.into_boxed_slice(),
            skylight: self.skylight,
        }
    }

    pub fn from_maybe_component(
        objs: Option<Vec<SceneObject>>,
        lights: Option<Vec<light::Lights>>,
        skylight: Option<Color3>,
    ) -> Self {
        SceneBuilder {
            objects: objs.unwrap_or_else(Vec::new),
            lights: lights.unwrap_or_else(Vec::new),
            skylight: skylight.unwrap_or_else(|| Color3::new(0.0, 0.0, 0.0)),
        }
    }
}

impl Default for SceneBuilder {
    fn default() -> Self {
        SceneBuilder {
            objects: Vec::new(),
            lights: Vec::new(),
            skylight: Color3::new(0.0, 0.0, 0.0),
        }
    }
}

use crate::rtracer::bvh::BVHTree;

use crate::rtracer::thread_buffer::ThreadBuffer;
use derive_more::{From, Into};
use typed_index_collections::TiSlice;

#[derive(Clone, From, Into, Debug)]
pub struct SceneObjectIndex(usize);

#[derive(Serialize)]
pub struct Scene {
    bounded_objects: Box<TiSlice<SceneObjectIndex, SceneObject>>,
    #[serde(skip)]
    bvh: BVHTree,
    unbounded_objects: Box<[SceneObject]>,
    lights: Box<[light::Lights]>,
    skylight: Color3,
}

impl Scene {
    pub fn direct_light_at(
        &self,
        point: Point3<f32>,
        normal: Unit<Vector3<f32>>,
        thread_buffer: &mut ThreadBuffer,
    ) -> Color3 {
        let direct_light = self
            .lights
            .iter()
            .map(|x| x.direct_light_at(point, normal, self, thread_buffer))
            .sum::<Color3>();

        direct_light // + self.skylight
    }

    #[inline]
    pub fn bounded(&self) -> &TiSlice<SceneObjectIndex, SceneObject> {
        self.bounded_objects.as_ref()
    }

    #[inline]
    pub fn unbounded(&self) -> &[SceneObject] {
        self.unbounded_objects.as_ref()
    }

    #[inline]
    pub fn bvh(&self) -> &BVHTree {
        &self.bvh
    }

    #[inline]
    pub fn lights(&self) -> &[light::Lights] {
        self.lights.as_ref()
    }

    #[inline]
    pub fn skylight(&self) -> &Color3 {
        &self.skylight
    }

    #[inline]
    pub fn skylight_mut(&mut self) -> &mut Color3 {
        &mut self.skylight
    }

    #[inline]
    pub fn get_skylight(&self) -> Color3 {
        self.skylight
    }
}
