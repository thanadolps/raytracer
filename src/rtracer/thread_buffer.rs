use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128Plus;
use rand::rngs::mock::StepRng;

// per thread variable
#[derive(Clone)]
pub struct ThreadBuffer {
    pub bvh_buffer: Vec<usize>,
    // pub rng: StepRng,
    pub rng: Xoroshiro128Plus
}

impl Default for ThreadBuffer {
    fn default() -> Self {
        ThreadBuffer {
            bvh_buffer: Vec::new(),
            rng: Xoroshiro128Plus::seed_from_u64(0),
            // rng: StepRng::new(2, 1)
        }
    }
}
