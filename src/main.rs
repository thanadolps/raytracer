use nalgebra::{Vector3, Point3, Rotation3, Unit, Similarity3, Translation3, UnitQuaternion, Matrix, MatrixMN,
			U3, U4, Matrix3x4, Vector2};

mod rtracer;
use rtracer::SceneObject;
use rtracer::geometric::{Sphere, InfinitePlane, Disc};
use rtracer::renderer::{render, RenderImage};
use rtracer::light::{PointLight, DirectionalLight, AreaLight};
use rtracer::Color3;
use std::time::Instant;
use std::f32::consts::FRAC_PI_4;

fn main() {
	// let (scene, camera) = rtracer::parser::parse_json();
	let (scene, camera) = setup();
	test_render(&scene, &camera);
}

fn setup() -> (rtracer::Scene<'static>, rtracer::Camera) {
	let camera = rtracer::Camera::new(
		Point3::new(0.0, 0.0, 0.0),
		Rotation3::from_euler_angles(0.0, 0.0, 0.0)
	);

	let sphere = SceneObject::new(Sphere {pos: Point3::new(2.0, 0.5, 0.5), radius: 0.6});
	let sphere2 = SceneObject::new(Sphere {pos: Point3::new(3.0, 0.0, 0.0), radius: 1.0});
	let floor = SceneObject::new(InfinitePlane {
		pos: Point3::new(1.0, 1.0, -1.0),
		norm: Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0))
	});
	let disc = SceneObject::new(Disc::new(
		Point3::new(1.0, 1.0, -1.0),
		Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
		5.0));
	let light = PointLight::new(Point3::new(0.0, 1.0, 0.5), Color3::new(1.0, 1.0, 1.0));
	let light2 = PointLight::new(Point3::new(0.0, -1.0, 0.5), Color3::new(1.0, 1.0, 1.0));
	let sun = DirectionalLight::new(
		Unit::new_normalize(Vector3::new(0.0, -1.0, -1.0)),
		Color3::new(0.1, 0.1, 0.1)
	);
	let area = AreaLight::new(
		Vector3::new(0.0, 1.0, 1.0),
		UnitQuaternion::from_euler_angles(0.0, 0.0, -FRAC_PI_4),
		0.5,
		None
	);

	let scene = rtracer::Scene::new(
		Some(vec![floor, sphere2]),
		Some(vec![
			// uncomment to enable light
			// Box::new(light),
			// Box::new(light2),
			// Box::new(sun),
			Box::new(area),
		]),
		None
	);

	(scene, camera)
}

fn test_render(scene: &rtracer::Scene, camera: &rtracer::Camera) {
	// render
	const IMAGE_SIZE: u32 = 250;
	const VIEWPORT_SIZE: u32 = 2;
	const UNIT_PER_PIXEL: f32 = VIEWPORT_SIZE as f32 / IMAGE_SIZE as f32;
	
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
