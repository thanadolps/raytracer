use crate::rtracer::geometric::Plane;
use crate::rtracer::helper::debug_normalize;
use crate::rtracer::light::AreaLight;
use crate::rtracer::Color3;
use nalgebra::{Point3, Similarity3, Unit, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use crate::rtracer::material::PBRDiffuse;

pub mod squared {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (data.sqrt()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        f32::deserialize(deserializer).map(|x| x * x)
    }
}

#[derive(Deserialize)]
pub struct PlaneProxy {
    pos: Point3<f32>,
    norm: Unit<Vector3<f32>>,
    span: Vector3<f32>,
    scale: Option<[f32; 2]>,
    cospan: Option<Vector3<f32>>,
}

impl From<PlaneProxy> for Plane {
    fn from(proxy: PlaneProxy) -> Self {
        let mut norm = proxy.norm;
        norm.renormalize();

        let scale = proxy.scale.unwrap_or([1.0, 1.0]);

        let cospan = proxy.cospan.unwrap_or_else(|| norm.cross(&proxy.span));

        let span = proxy.span * scale[0];
        let cospan = cospan * scale[1];

        let (span_dir, span_length) = Unit::new_and_get(span);
        let (cospan_dir, cospan_length) = Unit::new_and_get(cospan);

        Plane {
            pos: proxy.pos,
            norm,
            span_dir,
            span_length,
            cospan_dir,
            cospan_length,
        }
    }
}

#[derive(Deserialize)]
pub struct AreaLightProxy {
    pub plane: PlaneProxy,
    pub light: Color3,
}

impl From<AreaLightProxy> for AreaLight {
    fn from(proxy: AreaLightProxy) -> Self {
        AreaLight {
            plane: proxy.plane.into(),
            light: proxy.light,
        }
    }
}

#[derive(Deserialize)]
pub struct PBRDiffuseProxy {
    color: Color3,
    albedo: f32,
    iteration: usize,
}

impl From<PBRDiffuseProxy> for PBRDiffuse {
    fn from(proxy: PBRDiffuseProxy) -> Self {
        PBRDiffuse {
            color_albedo: proxy.albedo * proxy.color,
            iteration: proxy.iteration
        }
    }
}
/*
#[test]
fn a() {
    let x = AreaLightProxy {
        plane: ,
        light: Color3::new(0.0,0.0,0.0)
    };
    crate::utils::print_ron(&x);
}*/
