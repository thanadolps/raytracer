#[derive(Copy, Clone)]
pub struct RayCastInfo {
    ray_number: usize,
}

impl RayCastInfo {
    pub fn new() -> Self {
        RayCastInfo { ray_number: 0 }
    }

    pub fn increment_ray_number(&mut self) {
        self.ray_number += 1;
    }

    pub fn ray_depth(&self) -> usize {
        self.ray_number
    }
}
