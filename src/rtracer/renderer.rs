use std::cmp::Ordering::Equal;

use image::{ImageBuffer, Pixel};

use image::Rgb;
use itertools::Itertools;
use itertools::MinMaxResult::*;
use nalgebra::{Point3, Unit, Vector3};
use rand::prelude::Rng;
use rand::SeedableRng;

use crate::rtracer::{material::Material, RayCastInfo, SceneObject};

use super::Camera;
use super::Color3;
use super::HitInfo;

use super::scene::Scene;
use super::shape::Shape;

use crate::rtracer::geometric::Shapes;
use crate::rtracer::parser::ColorMapConfig;
use crate::rtracer::thread_buffer::ThreadBuffer;
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use std::ops::RangeBounds;

pub type RenderImage = ImageBuffer<Rgb<u8>, Vec<u8>>;
type RenderBuffer = ImageBuffer<Rgb<f32>, Vec<f32>>;

// TODO: extract per-ray render part for improving usability
pub fn render(
    scene: &Scene,
    camera: &Camera,
    image_size: u32,
    unit_per_pixel: f32,
    color_map_config: &ColorMapConfig,
) -> RenderImage {
    let mut img: RenderBuffer = ImageBuffer::new(image_size, image_size);
    let raycast_info = RayCastInfo::new();

    let half_width = img.width() / 2;
    let half_height = img.height() / 2;

    img.enumerate_pixels_mut().par_bridge().for_each_with(
        ThreadBuffer::default(),
        |thread_buffer, (px, py, pixel)| {
            // get ray from camera
            let ray_dir =
                camera.ray_at_pixel_position(px, py, unit_per_pixel, half_width, half_height);

            // raycast!
            let light =
                raycast_compute_light(scene, thread_buffer, camera.pos, ray_dir, raycast_info);
            *pixel = Rgb([light[0], light[1], light[2]]);
        },
    );

    // map from f32 image to u8 image
    color_map(
        img,
        color_map_config.v_min,
        color_map_config.v_max,
        color_map_config.gamma,
    )
    // color_map(img, None, None)
    // TODO: post process with dither and blur
}

pub fn raycast_compute_light(
    scene: &Scene,
    thread_buffer: &mut ThreadBuffer,
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    info: RayCastInfo,
) -> Color3 {
    let mut info = info.clone();

    if let Some((hit, obj_ref)) =
        raycast_return_ref(scene, origin, dir, &mut thread_buffer.bvh_buffer)
    {
        info.increment_ray_number();
        obj_ref
            .material
            .compute_light(scene, thread_buffer, &hit, &obj_ref, info)
    } else {
        scene.get_skylight()
    }
}

// TODO: maybe change to input array of Color3?
fn color_map(
    buf: RenderBuffer,
    vmin: Option<f32>,
    vmax: Option<f32>,
    gamma: Option<f32>,
) -> RenderImage {
    // calculate min, max of image
    let (min, max) = match (vmin, vmax) {
        (None, None) => match buf
            .iter()
            .minmax_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal))
        {
            NoElements => panic!("Blank Image Provided!"),
            OneElement(_a) => panic!("MonoColor Image Provided Without vmin, vmax"),
            MinMax(&a, &b) => (a, b),
        },
        (None, Some(max)) => (
            *buf.iter()
                .min_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal))
                .unwrap(),
            max,
        ),
        (Some(min), None) => (
            min,
            *buf.iter()
                .max_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal))
                .unwrap(),
        ),
        (Some(min), Some(max)) => (min, max),
    };

    // extract gamma
    let gamma = gamma.unwrap_or(2.5);
    let gamma_correction = gamma.recip();

    /*
    let hsv_image = ImageBuffer::from_fn(uf.width(), buf.height(), |i, j| {
        let inner_buf = img[(i, j)].0;
        let rgb = palette::rgb::Rgb::new(inner_buf[0],inner_buf[1],inner_buf[2]);
        let hsl = palette::Hsl::from(rgb);
        hsl
    });*/

    // copy and convert pixel from buf to img
    let mut img: RenderImage = ImageBuffer::new(buf.width(), buf.height());
    let map_fn = |v: f32| {
        let normalized = (v - min) / (max - min);
        (255.0 * normalized.powf(gamma_correction)) as u8
    };
    for (b, p) in buf.pixels().zip(img.pixels_mut()) {
        *p = Rgb([map_fn(b[0]), map_fn(b[1]), map_fn(b[2])]);
    }

    // blur an image a bit
    let img = image::imageops::blur(&img, 0.3);

    img
}

fn raycast_shapes<'a>(
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    shapes: impl Iterator<Item = &'a Shapes>,
) -> Option<HitInfo> {
    shapes
        .filter_map(|shape| shape.intersect(origin, dir))
        .filter(|x| x.dist > 1e-6)
        .min_by_key(|a| OrderedFloat(a.dist))
}

fn raycast_shapes_return_ref<'a>(
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    scene_objects: impl Iterator<Item = &'a SceneObject>,
) -> Option<(HitInfo, &'a SceneObject)> {
    scene_objects
        .map(|obj| (&obj.shape, obj))
        .filter_map(|(shape, obj)| Some((shape.intersect(origin, dir)?, obj)))
        .filter(|(x, _)| x.dist > 1e-6)
        .min_by_key(|(a, _)| OrderedFloat(a.dist))
}

fn raycast_bounded(
    scene: &Scene,
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    t_span: &impl RangeBounds<f32>,
    bvh_buffer: &mut Vec<usize>,
) -> Option<HitInfo> {
    let bounded = scene.bounded();
    let valid_obj = scene
        .bvh()
        .query_leaf(
            |node| !node.bounding_box.does_ray_hit(origin, dir, t_span),
            bvh_buffer,
        )
        // safe because bounded is monotonically increasing vector
        // TODO: type encode monotonically incresing vector
        .map(|index| &unsafe { bounded.get_unchecked(index) }.shape);
    raycast_shapes(origin, dir, valid_obj)
}

fn raycast_bounded_return_ref<'a>(
    scene: &'a Scene,
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    t_span: &impl RangeBounds<f32>,
    bvh_buffer: &mut Vec<usize>,
) -> Option<(HitInfo, &'a SceneObject)> {
    let bounded = scene.bounded();
    let valid_obj = scene
        .bvh()
        .query_leaf(
            |node| !node.bounding_box.does_ray_hit(origin, dir, t_span),
            bvh_buffer,
        )
        // safe because bounded is monotonically increasing vector
        .map(|index| unsafe { bounded.get_unchecked(index) });
    raycast_shapes_return_ref(origin, dir, valid_obj)
}

pub fn raycast(
    scene: &Scene,
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    bvh_buffer: &mut Vec<usize>,
) -> Option<HitInfo> {
    let unbounded_hit: Option<HitInfo> =
        raycast_shapes(origin, dir, scene.unbounded().iter().map(|obj| &obj.shape));

    // TODO: unchecked index
    let _bounded = scene.bounded();
    let _bvh = scene.bvh();
    if let Some(hit) = unbounded_hit {
        raycast_bounded(scene, origin, dir, &(0.0..hit.dist), bvh_buffer).or(Some(hit))
    } else {
        raycast_bounded(scene, origin, dir, &(..), bvh_buffer)
    }
}

pub fn raycast_return_ref<'a>(
    scene: &'a Scene,
    origin: Point3<f32>,
    dir: Unit<Vector3<f32>>,
    bvh_buffer: &mut Vec<usize>,
) -> Option<(HitInfo, &'a SceneObject)> {
    let unbounded_hit = raycast_shapes_return_ref(origin, dir, scene.unbounded().iter());

    // TODO: unchecked index
    if let Some(hit) = unbounded_hit {
        raycast_bounded_return_ref(scene, origin, dir, &(0.0..hit.0.dist), bvh_buffer).or(Some(hit))
    } else {
        raycast_bounded_return_ref(scene, origin, dir, &(..), bvh_buffer)
    }
}
