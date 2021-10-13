use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Deserializer, Serialize};

use custom_error::custom_error;

use super::{Camera, Scene};
use crate::rtracer::scene::SceneBuilder;

#[derive(Serialize, Deserialize)]
pub struct SceneData {
    #[serde(deserialize_with = "deserialize_scene")]
    pub scene: Scene,
    pub camera: Camera,
    pub config: RenderConfig,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ColorMapConfig {
    pub v_min: Option<f32>,
    pub v_max: Option<f32>,
    pub gamma: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct RenderConfig {
    pub image_size: u32,
    pub viewport_size: u32,
    pub output_file: PathBuf,
    #[serde(default)]
    pub color_map: ColorMapConfig,
}

custom_error! { pub SceneParserError
    RonSerdeError {source: ron::error::Error} = "Encounter error while (de)serializing scene data",
    IOError {source: io::Error} = "Encounter error while opening file"
}

pub fn load_scene_data(path: impl AsRef<Path>) -> Result<SceneData, SceneParserError> {
    let scene_data = ron::de::from_reader(fs::File::open(path)?)?;
    Ok(scene_data)
}

pub fn save_scene_data(path: impl AsRef<Path>, scene: &SceneData) -> Result<(), SceneParserError> {
    let encoded_scene = to_string_pretty(scene, PrettyConfig::default())?;
    fs::write(path, encoded_scene)?;
    Ok(())
}

fn deserialize_scene<'de, D>(deserializer: D) -> Result<Scene, D::Error>
where
    D: Deserializer<'de>,
{
    SceneBuilder::deserialize(deserializer).map(|b| b.build())
}

pub mod serde_interface {
    use nalgebra::{Point3, Rotation3};
    use serde::{Deserialize, Serialize};

    use super::super::camera::Camera;

    #[derive(Serialize, Deserialize)]
    pub struct CameraSerdeInterface {
        position: Point3<f32>,
        rotation: (f32, f32, f32),
    }

    impl From<CameraSerdeInterface> for Camera {
        fn from(inter: CameraSerdeInterface) -> Camera {
            Camera::new(
                inter.position,
                Rotation3::from_euler_angles(inter.rotation.0, inter.rotation.1, inter.rotation.2),
            )
        }
    }

    impl From<Camera> for CameraSerdeInterface {
        fn from(camera: Camera) -> CameraSerdeInterface {
            CameraSerdeInterface {
                position: camera.pos,
                rotation: camera.get_rotation().euler_angles(),
            }
        }
    }
}
