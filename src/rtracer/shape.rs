use nalgebra::{Point3, Unit, Vector3};

use enum_dispatch::enum_dispatch;

use super::HitInfo;
use crate::utils::aabb::AABB;

#[enum_dispatch]
pub trait Shape {
    fn intersect(&self, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo>;
    fn bounding_box(&self) -> Option<AABB>;
}

pub mod geometric {
    use nalgebra::{ComplexField, Point3, Unit, Vector3};
    use serde::{Deserialize, Serialize};

    use enum_dispatch::enum_dispatch;

    use super::HitInfo;
    use super::Shape;
    use crate::rtracer::helper::debug_normalize;
    use crate::utils::aabb::AABB;

    #[enum_dispatch(Shape)]
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Shapes {
        Sphere,
        InfinitePlane,
        Disc,
        Plane,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SphereProxy {
        pub pos: Point3<f32>,
        pub radius: f32,
    }

    impl From<SphereProxy> for Sphere {
        fn from(proxy: SphereProxy) -> Self {
            Sphere {
                pos: proxy.pos,
                radius: proxy.radius,
                radius_squared: proxy.radius * proxy.radius,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(from = "SphereProxy")]
    pub struct Sphere {
        pub pos: Point3<f32>,
        pub radius: f32,
        pub radius_squared: f32,
    }

    impl Shape for Sphere {
        fn intersect(&self, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo> {
            // http://viclw17.github.io/2018/07/16/raytracing-ray-sphere-intersection/
            // + optimization: a = 1 always (self dot = mag, mag of unit vector = 1)
            let dac = origin - self.pos;
            let half_b = dir.dot(&dac);
            let c: f32 = dac.magnitude_squared() - self.radius_squared;

            let discriminant = half_b * half_b - c;

            let dist = -half_b - discriminant.try_sqrt()?;

            // if behide camera
            if dist.is_sign_negative() {
                return None;
            }

            let intersection = origin.clone() + dir.into_inner() * dist;
            let normal = debug_normalize((intersection - self.pos) / self.radius);

            Some(HitInfo {
                incoming_dir: dir,
                dist,
                intersection,
                normal,
            })
        }

        fn bounding_box(&self) -> Option<AABB> {
            let r = self.radius;
            let r_vec = Vector3::new(r, r, r);
            Some(AABB::new_uncheck(self.pos - r_vec, self.pos + r_vec))
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct InfinitePlane {
        pub pos: Point3<f32>,
        pub norm: Unit<Vector3<f32>>,
    }

    impl InfinitePlane {
        fn _intersect(
            self_pos: Point3<f32>,
            self_norm: Unit<Vector3<f32>>,
            origin: Point3<f32>,
            dir: Unit<Vector3<f32>>,
        ) -> Option<HitInfo> {
            let deno = dir.dot(self_norm.as_ref());

            if deno > -1e-2 {
                return None;
            }

            let dist = ((self_pos - origin).dot(self_norm.as_ref())) / deno;

            if dist > 0.0 {
                Some(HitInfo {
                    incoming_dir: dir,
                    dist,
                    intersection: origin + dir.as_ref() * dist,
                    normal: self_norm,
                })
            } else {
                None
            }
        }
    }

    impl Shape for InfinitePlane {
        fn intersect(&self, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo> {
            InfinitePlane::_intersect(self.pos.clone(), self.norm, origin, dir)
        }

        fn bounding_box(&self) -> Option<AABB> {
            None
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Disc {
        pub pos: Point3<f32>,
        pub norm: Unit<Vector3<f32>>,
        #[serde(rename = "radius", with = "crate::utils::proxy_serialize::squared")]
        pub r_sq: f32,
    }

    impl Disc {
        pub fn new(pos: Point3<f32>, norm: Unit<Vector3<f32>>, r: f32) -> Self {
            Disc {
                pos,
                norm,
                r_sq: r * r,
            }
        }
    }

    impl Shape for Disc {
        fn intersect(&self, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo> {
            InfinitePlane::_intersect(self.pos.clone(), self.norm, origin, dir)
                .filter(|hit| (hit.intersection - self.pos).norm_squared() < self.r_sq)
        }

        // NOTE: incomplete implementation
        fn bounding_box(&self) -> Option<AABB> {
            let r = self.r_sq.sqrt();
            let span = Vector3::new(r, r, 0.001);
            Some(AABB::new(self.pos - span, self.pos + span))
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(from = "crate::utils::proxy_serialize::PlaneProxy")]
    // plane = t*span + u*cospan; (t,u) in [-1,1]^2
    pub struct Plane {
        pub pos: Point3<f32>,
        pub norm: Unit<Vector3<f32>>,
        pub span_dir: Unit<Vector3<f32>>,
        pub cospan_dir: Unit<Vector3<f32>>,
        pub span_length: f32,
        pub cospan_length: f32,
    }

    impl Shape for Plane {
        fn intersect(&self, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo> {
            InfinitePlane::_intersect(self.pos, self.norm, origin, dir).filter(|hit| {
                let displacement: Vector3<f32> = hit.intersection - self.pos;
                displacement.dot(&self.span_dir).abs() <= self.span_length
                    && displacement.dot(&self.cospan_dir).abs() <= self.cospan_length
            })
        }

        fn bounding_box(&self) -> Option<AABB> {
            // dbg!("ww");
            // return None;
            let total_span = self.span_dir.scale(self.span_length) + self.cospan_dir.scale(self.cospan_length) + Vector3::new(1e-3, 1e-3, 1e-3);
            Some(AABB::new(self.pos + total_span, self.pos - total_span))
        }
    }
}
