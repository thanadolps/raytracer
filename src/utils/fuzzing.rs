use assert_approx_eq::assert_approx_eq;
use nalgebra::{Unit, Vector3};
use proptest::prelude::*;

pub fn vector_strategy() -> impl Strategy<Value = Vector3<f32>> {
    let s: proptest::num::f32::Any = any::<f32>();
    [s.clone(); 3].prop_map(|a| Vector3::from(a))
}

pub fn unit_vector_strategy() -> impl Strategy<Value = Unit<Vector3<f32>>> {
    let z = -1f32..=1.0;
    let angle = 0f32..(core::f32::consts::TAU);
    (angle, z).prop_map(|(theta, z)| {
        let (s, c) = theta.sin_cos();
        let k = (1.0 - z * z).sqrt();
        let vec = Vector3::new(k * s, k * c, z);
        assert_approx_eq!(vec.magnitude_squared(), 1.0);
        Unit::new_unchecked(vec)
    })
}
