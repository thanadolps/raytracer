use std::collections::Bound;
use std::fmt::{Debug, Formatter};
use std::ops::RangeBounds;

use nalgebra::{Point3, Unit, Vector3};

#[derive(Clone, PartialEq)]
pub struct AABB {
    min: Point3<f32>,
    max: Point3<f32>,
}

impl AABB {
    pub fn new(mut a: Point3<f32>, mut b: Point3<f32>) -> Self {
        a.iter_mut().zip(b.iter_mut()).for_each(|(i, j)| {
            if i > j {
                std::mem::swap(i, j);
            }
        });
        AABB::new_uncheck(a, b)
    }

    pub fn new_uncheck(min: Point3<f32>, max: Point3<f32>) -> Self {
        AABB { min, max }
    }

    pub fn from_floats(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> Self {
        AABB::new(Point3::new(ax, ay, az), Point3::new(bx, by, bz))
    }

    pub fn does_ray_hit(
        &self,
        ray_origin: Point3<f32>,
        ray_dir: Unit<Vector3<f32>>,
        t_boundary: &impl RangeBounds<f32>,
    ) -> bool {
        self.calc_ray_hit_span(ray_origin, ray_dir, t_boundary)
            .is_some()
    }

    pub fn calc_ray_hit_span(
        &self,
        ray_origin: Point3<f32>,
        ray_dir: Unit<Vector3<f32>>,
        t_boundary: &impl RangeBounds<f32>,
    ) -> Option<[f32; 2]> {
        let t_min = match t_boundary.start_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => f32::NEG_INFINITY,
        };
        let t_max = match t_boundary.end_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => f32::INFINITY,
        };

        (0..3).try_fold([t_min, t_max], |[mut t_min, mut t_max], i| {
            let inv_d = ray_dir[i].recip();
            let mut t0 = (self.min[i] - ray_origin[i]) * inv_d;
            let mut t1 = (self.max[i] - ray_origin[i]) * inv_d;
            if inv_d.is_sign_negative() {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = f32::max(t0, t_min);
            t_max = f32::min(t1, t_max);
            (t_min < t_max).then(|| [t_min, t_max])
        })
    }

    pub fn contain(&self, p: Point3<f32>, tor: f32) -> bool {
        (0..3).all(|i| ((self.min[i] - tor)..=(self.max[i] + tor)).contains(&p[i]))
    }

    pub fn union(&self, other: &AABB) -> AABB {
        // FIXME: doesn't work for some reason (simd problem?)
        let mut min = self.min;
        min.iter_mut().zip(other.min.iter()).for_each(|(a, b)| {
            if b < a {
                *a = *b
            }
        });
        let mut max = self.max;
        max.iter_mut().zip(other.max.iter()).for_each(|(a, b)| {
            if b > a {
                *a = *b
            }
        });

        AABB { min, max }
    }

    pub fn min(&self) -> &Point3<f32> {
        &self.min
    }

    pub fn max(&self) -> &Point3<f32> {
        &self.max
    }
}

impl Debug for AABB {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "AABB({}..{})", self.min.xyz(), self.max.xyz())
    }
}

#[cfg(test)]
mod tests {

    use proptest::test_runner::TestRunner;

    use crate::utils::aabb::AABB;
    use crate::utils::*;
    use nalgebra::Point3;

    #[test]
    fn construction() {
        let aabb = AABB::new(Point3::new(-1.0, 5.0, 2.0), Point3::new(1.0, -10.0, -4.0));
        assert_eq!(aabb.min, Point3::new(-1.0, -10.0, -4.0));
        assert_eq!(aabb.max, Point3::new(1.0, 5.0, 2.0));
    }

    #[test]
    fn order_invariant() {
        let mut runner = TestRunner::default();
        let span = [-100.0_f32..100.0, -100.0_f32..100.0, -100.0_f32..100.0];
        runner
            .run(&(span.clone(), span), |(a, b)| {
                let aabb = AABB::new(Point3::from(a), Point3::from(b));
                assert!(aabb.min < aabb.max);
                Ok(())
            })
            .unwrap();
    }
    /*
        #[test]
        fn valid_hit_fuzzing() {
            let mut runner = TestRunner::default();
            let o = fuzzing::vector_strategy();
            let d = fuzzing::unit_vector_strategy();
            let a = fuzzing::vector_strategy();
            let b = fuzzing::vector_strategy();

            let hs = (o, d, a, b).prop_filter_map("not hit", |(o, d, a, b)| {
                let aabb = AABB::new(Point3::from(a), Point3::from(b));
                let o = Point3::from(o);
                let t = aabb.calc_ray_hit_span(o, d, ..)?;
                Some((aabb, t, o, d))
            });

            let ep = 0.5;

            runner
                .run(&hs, |(aabb, [t0, t1], o, d)| {
                    let md = (t1 - t0) / 10.0;
                    for i in 1..10 {
                        assert!(aabb.contain(o + d.into_owned() * (t0 + (i as f32) * md), 0.01));
                    }
                    if !aabb.contain(o, 0.01) {
                        assert!(!aabb.contain(o + d.into_owned() * (t0 - ep), 0.01));
                        assert!(!aabb.contain(o + d.into_owned() * (t1 + ep), 0.01));
                    }
                    Ok(())
                })
                .unwrap()
        }
    */
    /*
        #[test]
        fn simple_unbounded_test() {
            let aabb = AABB::new(Point3::new(1.0, -0.5, -0.5), Point3::new(2.0, 1.0, 0.1));
            assert_eq!(
                aabb.does_ray_hit(
                    Point3::new(0.0, 0.0, 0.0),
                    Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)),
                    ..
                ),
                true
            );
            assert_eq!(
                aabb.does_ray_hit(
                    Point3::new(0.0, 0.0, 0.0),
                    Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                    ..
                ),
                false
            );
            assert_eq!(
                aabb.does_ray_hit(
                    Point3::new(0.0, 0.0, 0.0),
                    Unit::new_normalize(Vector3::new(1.0, 0.99, 0.0)),
                    ..
                ),
                true
            );
            assert_eq!(
                aabb.does_ray_hit(
                    Point3::new(0.0, 0.0, 0.0),
                    Unit::new_normalize(Vector3::new(1.0, 1.01, 0.0)),
                    ..
                ),
                false
            );
        }

        #[test]
        fn boundary_test() {
            let aabb = AABB::new(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 0.0, 1.0));
            assert_eq!(
                aabb.does_ray_hit(
                    Point3::new(0.0, -2.0, 0.0),
                    Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                    ..
                ),
                false
            );
        }
    */
    #[test]
    fn union_test() {
        let a = AABB::from_floats(0.0, 1.0, -1.0, 2.0, 5.0, 0.0);
        let b = AABB::from_floats(1.0, 0.0, 0.0, 2.0, 1.0, 2.0);
        assert_eq!(
            a.union(&b),
            AABB::from_floats(0.0, 0.0, -1.0, 2.0, 5.0, 2.0)
        );
    }
}
