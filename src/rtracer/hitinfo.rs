use nalgebra::{Point3, Unit, Vector3};

// TODO: include more info such as material/ objectId, etc..
pub struct HitInfo {
    pub incoming_dir: Unit<Vector3<f32>>,
    pub dist: f32,
    pub intersection: Point3<f32>,
    pub normal: Unit<Vector3<f32>>,
}
