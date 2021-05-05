use std::f32::consts::FRAC_PI_4;
use std::time::Instant;

use nalgebra::{Point3, Rotation3, Unit, UnitQuaternion, Vector3};

use rtracer::Color3;
use rtracer::geometric::{InfinitePlane, Sphere};
use rtracer::light::{AreaLight};
use rtracer::renderer::{render, RenderImage};
use rtracer::SceneObject;

use crate::rtracer::SceneData;

mod rtracer;

fn main() {
	// let scene_data = setup();
	// rtracer::parser::save_scene_data("data.ron", &scene_data).unwrap();
	// test();
	let scene=  rtracer::parser::load_scene_data("data.ron").unwrap();
	test_render(&scene);
}

fn test() {



	let q2: UnitQuaternion<f64> =
		UnitQuaternion::from_axis_angle(&Vector3::x_axis(), std::f64::consts::PI/4.0);

	// println!("{}", (q2 * Vector3::y_axis()).into_inner());




	/*let k = &mut u;
	let l: Unit<Vector3<f64>> = (q2 * *k);
	println!("{}", l.into_inner());

	*k = l;
	println!("{}", k.into_inner());*/

	// let mut u: Unit<Vector3<f64>> = Vector3::y_axis();
	(0..8).scan(Vector3::y_axis(), |q: &mut Unit<Vector3<f64>>, _| {
		*q = q2 * *q;
		Some(*q)
	}).for_each(|x| println!("{}", x.into_inner()));
}

fn setup() -> SceneData {

	use crate::rtracer::material;

	let camera = rtracer::Camera::new(
		Point3::new(0.0, 0.0, 0.0),
		Rotation3::from_euler_angles(0.0, 0.0, 0.0)
	);

	let sphere =
        SceneObject::new(
            Sphere {pos: Point3::new(3.0, 2.0, 0.0), radius: 0.5},
            // material::Diffuse::new([0.5, 1.0, 0.5].into()),
            material::PerfectReflective::new([0.5, 0.5, 1.0].into())
        );
	let sphere2 =
		SceneObject::new(
			Sphere {pos: Point3::new(3.0, 0.0, 0.0), radius: 1.0},
			material::Diffuse::new([1.0, 0.5, 0.5].into()),
            //material::PerfectReflective,
		);
	let floor =
		SceneObject::new(
			InfinitePlane {
				pos: Point3::new(1.0, 1.0, -1.0),
				norm: Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0))
			},
			// material::Reflective::new(0.5, 3)
		        material::Diffuse::new([0.5, 0.5, 1.0].into())
        );/*
	let disc = SceneObject::new(Disc::new(
		Point3::new(1.0, 1.0, -1.0),
		Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
		5.0));
	let light = PointLight::new(Point3::new(0.0, 1.0, 0.5), Color3::new(1.0, 1.0, 1.0));
	let light2 = PointLight::new(Point3::new(0.0, -1.0, 0.5), Color3::new(1.0, 1.0, 1.0));
	let sun = DirectionalLight::new(
		Unit::new_normalize(Vector3::new(0.0, -1.0, -1.0)),
		Color3::new(0.1, 0.1, 0.1)
	);*/
	let area = AreaLight::new(
		Vector3::new(0.0, 1.0, 1.0),
		UnitQuaternion::from_euler_angles(0.0, 0.0, -FRAC_PI_4),
		0.5,
		None
	);

	let scene = rtracer::Scene::from_maybe_component(
		Some(vec![sphere, floor, sphere2]),
		Some(vec![
			// uncomment to enable light
			// light.into(),
			// light2.into(),
			// sun.into(),
			area.into(),
		]),
		Some(Color3::new(0.05, 0.07, 0.1))
	);

	SceneData {scene, camera}
}

fn test_render(scene_data: &rtracer::SceneData) {
	// render
	const IMAGE_SIZE: u32 = 500;
	const VIEWPORT_SIZE: u32 = 2;
	const UNIT_PER_PIXEL: f32 = VIEWPORT_SIZE as f32 / IMAGE_SIZE as f32;

	let (scene, camera) = (&scene_data.scene, &scene_data.camera);

	print!("Start Rendering...");
	let start_time = Instant::now();
	let rendered_image: RenderImage = render(scene, camera, IMAGE_SIZE, UNIT_PER_PIXEL);
	let duration = start_time.elapsed();
	println!("\nRendering Finish In {:.2}s", duration.as_secs_f32());
	
	// save
	const FILE_NAME: &str = "render.png";
	rendered_image.save(FILE_NAME).expect("Unable to save Image");
	println!("Saving to {}", FILE_NAME);
}

