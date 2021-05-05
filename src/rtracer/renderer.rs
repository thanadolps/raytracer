use std::cmp::Ordering::Equal;

use image::ImageBuffer;


use image::Rgb;
use itertools::Itertools;
use itertools::MinMaxResult::*;
use nalgebra::{Point3, Unit, Vector3};
use rand::prelude::{Rng, SmallRng};
use rand::SeedableRng;

use crate::rtracer::{material::Material, RayCastInfo, SceneObject};

use super::Camera;
use super::Color3;
use super::HitInfo;

use super::scene::Scene;
use super::shape::Shape;

pub type RenderImage = ImageBuffer<Rgb<u8>, Vec<u8>>;
type RenderBuffer = ImageBuffer<Rgb<f32>, Vec<f32>>;

// TODO: extract per-ray render part for improving reusability
pub fn render(scene: &Scene, camera: &Camera, image_size: u32, unit_per_pixel: f32) -> RenderImage {
	
	let mut img: RenderBuffer = ImageBuffer::new(image_size, image_size);
	let mut rng = SmallRng::from_entropy();
	let raycast_info = RayCastInfo::new();

	let half_width = img.width()/2;
	let half_height = img.height()/2;
	
	for (px, py, pixel) in img.enumerate_pixels_mut() {
		// get ray from camera
		let ray_dir =
			camera.ray_at_pixel_position(px, py, unit_per_pixel, half_width, half_height);
		
		// raycast!
		let light = raycast_compute_light(scene, camera.pos, ray_dir, raycast_info, &mut rng);
		*pixel = Rgb([light[0], light[1], light[2]]);
	}
	
	// map from f32 image to u8 image
	color_map(img, Some(0.0), None)
	// color_map(img, None, None)
	// TODO: post process with dither and blur
}

pub fn raycast_compute_light(
	scene: &Scene,
	origin: Point3<f32>,
	dir: Unit<Vector3<f32>>,
	info: RayCastInfo,
	rng: &mut impl Rng)
	-> Color3 {
	let mut info = info.clone();
	info.increment_ray_number();

	if let Some((hit, obj_ref)) = raycast_return_ref(scene, origin, dir) {
		// TODO: make this dependent on material
		obj_ref.material.compute_light(&scene, &hit, &obj_ref, info, rng)
	}
	else {
		scene.get_skylight()
	}
}


// TODO: maybe change to input array of Color3?
fn color_map(buf: RenderBuffer, vmin: Option<f32>, vmax: Option<f32>) -> RenderImage {
	
	// calculate min, max of image
	let (min, max) = match (vmin, vmax) {
		(None, None) =>
			match buf.iter().minmax_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal)) {
				NoElements => panic!("Blank Image Provided!"),
				OneElement(_a) => panic!("MonoColor Image Provided Without vmin, vmax"),
				MinMax(&a, &b) => (a, b),
			},
		(None, Some(max)) => (*buf.iter().min_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal)).unwrap(), max),
		(Some(min), None) => (min, *buf.iter().max_by(|a, b| a.partial_cmp(&b).unwrap_or(Equal)).unwrap()),
		(Some(min), Some(max)) => (min, max),
	};
	
	// copy and convert pixel from buf to img
	let mut img: RenderImage = ImageBuffer::new(buf.width(), buf.height());
	for (b, p) in buf.pixels().zip(img.pixels_mut()) {
		*p = Rgb([
			(255.0*(b[0] - min)/(max-min)) as u8,
			(255.0*(b[1] - min)/(max-min)) as u8,
			(255.0*(b[2] - min)/(max-min)) as u8
		]);
	}
	
	img
}

pub fn raycast(scene: &Scene, origin: Point3<f32>, dir: Unit<Vector3<f32>>) -> Option<HitInfo> {
	scene
		.iter_obj()
		.filter_map(|x| x.shape.intersect(origin, dir))
		.filter(|x| x.dist > 1e-6)
		.map(|x| x)
		.min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(Equal))
}


pub fn raycast_return_ref(scene: &Scene, origin: Point3<f32>, dir: Unit<Vector3<f32>>)
						  -> Option<(HitInfo, &SceneObject)> {
	scene
		.iter_obj()
		.filter_map(|x| x.shape.intersect(origin, dir).map(|k| (k, x)))
		.filter(|(x, _a): &(HitInfo, _)| x.dist > 1e-6)
		.min_by(|(a, _), (b, _)|
			a.dist.partial_cmp(&b.dist).unwrap_or(Equal)
		)
}