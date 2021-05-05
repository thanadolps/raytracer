
use std::ops::{RangeInclusive};

use nalgebra::{Unit, Vector3};
use num_traits::real::Real;

use assert_approx_eq::assert_approx_eq;

pub fn calculate_reflect_ray(incoming_ray: &Unit<Vector3<f32>>, normal: &Unit<Vector3<f32>>)
    -> Unit<Vector3<f32>> {
    // http://paulbourke.net/geometry/reflected/
    let ri = incoming_ray.into_inner();
    let ray = ri - 2.0 * ri.dot(normal.as_ref()) * normal.into_inner();
    debug_normalize(ray)
}

pub fn debug_normalize(v: Vector3<f32>) -> Unit<Vector3<f32>> {
    if cfg!(debug_assertions) {
        let (unit_vec, magnitude) = Unit::new_and_get(v);
        assert_approx_eq!(magnitude, 1.0, 1e-6);
        unit_vec
    }
    else {
        // dbg!(v);
        Unit::new_unchecked(v)
    }
}

pub fn map_float<F: Real>(x: F, src: RangeInclusive<F>, dest: RangeInclusive<F>) -> F {

    let dest_size = *dest.start() - *dest.end();
    let src_size = *src.start() - *src.end();

    *dest.start() + (dest_size/src_size) * (x - *src.start())
}

/*
type UnitSphereInnerIterator<
    F1: FnMut(&mut UV3, usize) -> Option<UV3>,
    F2: FnMut(&mut UV3, usize) -> Option<UV3>,
    F3: Fn(UV3) -> Scan<Range<usize>, UV3, F2>
> =
FlatMap<
    Scan<
        Range<usize>,
        UV3,
        F1
    >,
    Scan<
        Range<usize>,
        UV3,
        F2
    >,
    F3
>;

pub struct UnitSphereSurfaceIterator<F1, F2, F3>
where   F1: FnMut(&mut UV3, usize) -> Option<UV3>,
        F2: FnMut(&mut UV3, usize) -> Option<UV3>,
        F3: Fn(UV3) -> Scan<Range<usize>, UV3, F2>,
{
    qx: UnitQuaternion<f32>,
    qz: UnitQuaternion<f32>,
    inner_iterator: UnitSphereInnerIterator<F1, F2, F3>,
}

impl<F1, F2, F3> UnitSphereSurfaceIterator<F1, F2, F3>
where   F1: FnMut(&mut UV3, usize) -> Option<UV3>,
        F2: FnMut(&mut UV3, usize) -> Option<UV3>,
        F3: Fn(UV3) -> Scan<Range<usize>, UV3, F2>, {

}

impl<F1, F2, F3> Iterator for UnitSphereSurfaceIterator<F1, F2, F3>
where   F1: FnMut(&mut UV3, usize) -> Option<UV3>,
        F2: FnMut(&mut UV3, usize) -> Option<UV3>,
        F3: Fn(UV3) -> Scan<Range<usize>, UV3, F2>, {
    type Item = Unit<Vector3<f32>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iterator.next()
    }
}*/
/*
/// return iterator yielding uniform unit vector on unit sphere surface
///
/// Number of unit vector = resolution * resolution
pub fn unit_sphere_surface_iterator<F1, F2, F3> (resolution: usize) -> UnitSphereSurfaceIterator<F1, F2, F3>
where   F1: FnMut(&mut UV3, usize) -> Option<UV3>,
        F2: FnMut(&mut UV3, usize) -> Option<UV3>,
        F3: Fn(UV3) -> Scan<Range<usize>, UV3, F2>
{

    use nalgebra::UnitQuaternion;

    let qx: UnitQuaternion<f32> =
        UnitQuaternion::from_axis_angle(&Vector3::x_axis(), 2.0 * std::f32::consts::PI/resolution as f32);
    let qz: UnitQuaternion<f32> =
        UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 2.0 * std::f32::consts::PI/resolution as f32);

    let iterator =
        (0..resolution)
            .scan(Vector3::y_axis(), |uy: &mut Unit<Vector3<f32>>, _| {
                *uy = qx * *uy; // rotate unit vector on x axis every iteration
                Some(*uy)  // yield the rotated vector
            }) // iterator yield unit vector rotating around x-axis, end when completed 1 revolution
            .flat_map(|v|
                (0..resolution).scan(v, |uxy, _| {
                    *uxy = qz * *uxy; // rotate unit vector on z axis every iteration
                    Some(*uxy)  // yield that rotated vector
                })
            );

    UnitSphereSurfaceIterator {qx, qz, inner_iterator: iterator}
}*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_float_test() {
        assert_eq!(map_float(0.0, -1.0..=1.0, 0.0..=1.0), 0.5);
        assert_eq!(map_float(1.0, -1.0..=1.0, 0.0..=1.0), 1.0);
        assert_eq!(map_float(-1.0, -1.0..=1.0, 0.0..=1.0), 0.0);
        assert_eq!(map_float(0.0, 0.0..=1.0, -1.0..=1.0), -1.0);
    }
}