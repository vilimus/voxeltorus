use std::f32::consts::PI;
use macroquad::prelude::*;
use macroquad::rand::{rand, gen_range};
use rayon::prelude::*;
// Settings
const RESOLUTION: (f32, f32) = (800 as f32, 600 as f32);
const SCREEN: (usize, usize) = (400, 250);
const WORLDSIZE: [usize; 3] = [256, 256, 256];
const MOVEMENT_SPEED: f32 = 0.5;
const ROTATION_SPEED: (f32, f32) = (0.5, 0.5);
const FOV: (f32, f32) = (PI/2.0, PI/2.0*(SCREEN.1 as f32)/(SCREEN.0 as f32));
const VIEW_DISTANCE: usize = 128;
const TOUCH_DISTANCE: usize = 16;
const AMBIENT: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);
const RECTSIZE_X: f32 = RESOLUTION.0 / (SCREEN.0 as f32);
const RECTSIZE_Y: f32 = RESOLUTION.1 / (SCREEN.1 as f32);
struct Camera {
	i: usize,
	position: Vec3,
	angle: Vec2,
	movement_speed: f32,
	rotation_speed: (f32, f32),
	fov: (f32, f32),
	screen: (usize, usize),
}
#[derive(Copy, Clone)]
struct Voxel {
	color: Vec4,
	brightness: f32,
}
type Neighbors = [usize; 6];
type World = Vec<(Voxel, Neighbors)>;
fn furl(i: usize, j: usize, k: usize, ny: usize, nz: usize) -> usize {
	return i*ny*nz + j*nz + k
}
fn randf() -> f32 {
	(rand() as f32) / (u32::MAX as f32)
}
fn randr(a: f32, b: f32) -> f32 {
	a + (b -a) * randf()
}
fn combine_worlds(w1: World, w2: World) -> World {
	// add them as lists
	return w1;
}
fn build_world(nx: usize, ny: usize, nz: usize) -> World {
	let voxel = Voxel {
		color: vec4(0.0, 0.0, 0.0, 0.0),
		brightness: 0.0,
	};
	let mut world: World = vec![(voxel, [0, 0, 0, 0, 0, 0]); nx*ny*nz];
	for i in 0..nx {
		for j in 0..ny {
			for k in 0..nz {
				let n = furl(i, j, k, ny, nz);
				world[n].1 = [
					furl((i as i32 + 1).rem_euclid(nx as i32) as usize, j, k, ny, nz),
					furl((i as i32 - 1).rem_euclid(nx as i32) as usize, j, k, ny, nz),
					furl(i, (j as i32 + 1).rem_euclid(ny as i32) as usize, k, ny, nz),
					furl(i, (j as i32 - 1).rem_euclid(ny as i32) as usize, k, ny, nz),
					furl(i, j, (k as i32 + 1).rem_euclid(nz as i32) as usize, ny, nz),
					furl(i, j, (k as i32 - 1).rem_euclid(nz as i32) as usize, ny, nz)
				];
			}
		}
	}
	for i in 0..nx {
		for j in 0..16 {
			for k in 0..nz {
				let n = furl(i, j, k, ny, nz);
				world[n].0.color = vec4(
					randr(0.3, 0.4),
					randr(0.8, 0.9),
					randr(0.3, 0.4),
					gen_range(0, 2) as f32
				);
			}
		}
	}
	for _ in 0..200 {
		let i = gen_range(0, world.len());
		if world[i].0.color.w == 1.0 {
			world[i].0.color = vec4(1.0, 1.0, 1.0, 1.0);
			world[i].0.brightness = 1.0;
		}
	}
	return world;
}
fn lattice_intersect(pos: Vec3, v: Vec3) -> (Vec3, [i32; 3], f32) {
	let t = ((v.signum() + 1.0) / 2.0 - pos) / v;
	let t_min: f32 = t.min_element();
	let mut i_min: usize = 0;
	for i in 0..3 {
		if t[i] == t_min {
			i_min = i
		}
	}
	let mut key: [i32; 3] = [0, 0, 0];
	key[i_min] = v[i_min].signum() as i32;
	let key2 = vec3(key[0] as f32, key[1] as f32, key[2] as f32);
	let x_new = pos + t_min*v - key2;
	return (x_new, key, (t_min*v).length());
}
fn raycast(world: &World, vox_id: usize, basepoint: Vec3, ray: Vec3, max_steps: usize) -> (usize, Vec3, f32) {
	let (mut i, mut x) = (vox_id, basepoint);
	let mut k: [i32; 3];
	let mut dist = 0.0;
	let mut dt = 0.0;
	for step in 0..max_steps {
		(x, k, dt)  = lattice_intersect(x, ray);
		dist = dist + dt;
		if k[0] == 1 {
			i = world[i].1[0];
		} else if k[0] == -1 {
			i = world[i].1[1];
		} else if k[1] == 1 {
			i = world[i].1[2];
		} else if k[1] == -1 {
			i = world[i].1[3];
		} else if k[2] == 1 {
			i = world[i].1[4];
		} else if k[2] == -1 {
			i = world[i].1[5];
		}
		if world[i].0.color.w == 1.0 {
			return (i, x, dist);
		}
	}
	return (i, x, max_steps as f32);
}
fn update(world: &mut World, delta_t: f32) {
	for _ in 0..((delta_t * (world.len() as f32)) as usize)  {
		let i = gen_range(0, world.len() - 1);
		let mut neighbor_sum = vec4(0.0, 0.0, 0.0, 0.0);
		for j in world[i].1 {
			neighbor_sum = neighbor_sum + world[j].0.color;
		}
		if world[i].0.color.w == 1.0 && 4.0 <= neighbor_sum.w && neighbor_sum.w <= 6.0 {
			world[i].0.color.w = 1.0;
		} else if world[i].0.color.w < 1.0 && 3.0 <= neighbor_sum.w && neighbor_sum.w <= 6.0 {
			world[i].0.color.w = 1.0;
		} else {
			world[i].0.color.w = 0.0;
		}
	}
}
fn update_brightness(world: &mut World) {
	for i in 0..world.len() {
		if world[i].0.color.w < 1.0 {
			let mut neighbor_sum = 0.0;
			for j in world[i].1 {
				if world[j].0.brightness > neighbor_sum {
					neighbor_sum = world[j].0.brightness;
				}
			}
			world[i].0.brightness = neighbor_sum - 0.02;
			if world[i].0.brightness < 0.0 {
				world[i].0.brightness = 0.0;
			}
		}
	}
}
#[macroquad::main("VoxelTorus")]
async fn main() {
	request_new_screen_size(RESOLUTION.0, RESOLUTION.1);
	let mut world = build_world(WORLDSIZE[0], WORLDSIZE[1], WORLDSIZE[2]);
	//update(&mut world, 10.0);
	//for _ in 0..20 {
	//	update_brightness(&mut world);
	//}
	next_frame().await;
	let mut camera = Camera {
		i: 0,
		position: vec3(0.5, 0.5, 0.5),
		angle: vec2(0.0, 0.0),
		movement_speed: MOVEMENT_SPEED,
		rotation_speed: ROTATION_SPEED,
		fov: FOV,
		screen: SCREEN,
	};
	let mut screen: Vec<Vec<Vec4>> = vec![vec![vec4(0.0, 0.0, 0.0, 0.0); camera.screen.1]; camera.screen.0];
	let mut grabbed = true;
	while world[camera.i].0.color[3] == 1.0 {
		camera.i = world[camera.i].1[2];
	}
	let mut selected = Voxel {
		color: vec4(1.0, 1.0, 1.0, 1.0),
		brightness: 0.8,
	};
	loop {
		if is_mouse_button_released(MouseButton::Left) {
			grabbed = true;
		}
		if is_key_down(KeyCode::Escape) {
			grabbed = false;
		}
		set_cursor_grab(grabbed);
		show_mouse(!grabbed);
		let mut mouse_delta = vec2(0.0, 0.0);
		if grabbed {
			mouse_delta = mouse_delta_position();
		}
		camera.angle = camera.angle - vec2(camera.rotation_speed.0 * mouse_delta.x, -camera.rotation_speed.1 * mouse_delta.y);
		camera.angle[1] = clamp(camera.angle[1], -PI/2.0, PI/2.0);
		let look  = vec3( camera.angle[0].cos()*camera.angle[1].cos(), camera.angle[1].sin(),  camera.angle[0].sin()*camera.angle[1].cos());
		let up	= vec3(-camera.angle[0].cos()*camera.angle[1].sin(), camera.angle[1].cos(), -camera.angle[0].sin()*camera.angle[1].sin());
		let right = vec3(-camera.angle[0].sin(),					   0.0,					camera.angle[0].cos());
		let mut dx = vec3(0.0, 0.0, 0.0);
		if is_key_down(KeyCode::E) {
			dx = dx + vec3(0.0, 1.0, 0.0)
		}
		if is_key_down(KeyCode::Q) {
			dx = dx - vec3(0.0, 1.0, 0.0)
		}
		if is_key_down(KeyCode::W) {
			dx = dx + look;
		}
		if is_key_down(KeyCode::S) {
			dx = dx - look;
		}
		if is_key_down(KeyCode::A) {
			dx = dx - right;
		}
		if is_key_down(KeyCode::D) {
			dx = dx + right;
		}
		match dx.try_normalize() {
			Some(dx) => {
				camera.position = camera.position + camera.movement_speed * dx;
			},
			None => {},
		}
		let neighbors = world[camera.i].1;
		let mut camera_delta = vec3(0.0, 0.0, 0.0);
		if camera.position[0] < 0.0 {
			camera.i = neighbors[1];
			camera_delta[0] = 1.0;
		} else if camera.position[0] > 1.0 {
			camera.i = neighbors[0];
			camera_delta[0] = -1.0;
		} else if camera.position[1] < 0.0 {
			camera.i = neighbors[3];
			camera_delta[1] = 1.0;
		} else if camera.position[1] > 1.0 {
			camera.i = neighbors[2];
			camera_delta[1] = -1.0;
		} else if camera.position[2] < 0.0 {
			camera.i = neighbors[5];
			camera_delta[2] = 1.0;
		} else if camera.position[2] > 1.0 {
			camera.i = neighbors[4];
			camera_delta[2] = -1.0;
		}
		camera.position = camera.position + camera_delta;
		let (target_i, target_x, _) = raycast(&world, camera.i, camera.position, look, TOUCH_DISTANCE);
		if is_mouse_button_pressed(MouseButton::Left) {
			world[target_i].0.color.w = 0.0;
		}
		if is_mouse_button_pressed(MouseButton::Middle) {
			if world[target_i].0.color.w == 1.0 {
				selected = world[target_i].0;
			}
		}
		if is_mouse_button_pressed(MouseButton::Right) {
			if world[target_i].0.color.w == 1.0 {
				let (i, _, _) = raycast(&world, target_i, target_x, -look, 1);
				world[i].0.color = selected.color;
				world[i].0.brightness = selected.brightness;
			}
		}
		// update_brightness(&mut world);
		screen.par_iter_mut().enumerate().for_each(|(i, row)| {
			row.par_iter_mut().enumerate().for_each(|(j, cell)| {
				let right_coeff = (((i as f32) / (camera.screen.0 as f32) - 0.5) * camera.fov.0).atan();
				let up_coeff = (((j as f32) / (camera.screen.1 as f32) - 0.5) * camera.fov.1).atan();
				let ray = look + right_coeff*right - up_coeff*up;
				let (rayhit_i, rayhit_x, distance) = raycast(&world, camera.i, camera.position, ray, VIEW_DISTANCE);
				let mut fade = 0.0;
				let mut fade = 1.7321 * distance / (VIEW_DISTANCE as f32);// * 3.0.pow(-1.0/3.0);
				if rayhit_i == target_i {
					fade = 0.5*(fade + 1.0);
				}
				let (shortstop_i, _, _) = raycast(&world, rayhit_i, rayhit_x, -ray, 1);
				let local_brightness = clamp(world[shortstop_i].0.brightness, 0.0, 1.0);
				let color = (1.0 - local_brightness)*fade*AMBIENT + local_brightness*(1.0 - fade)*world[rayhit_i].0.color;
				*cell = color;
			})
		});
		screen.iter().enumerate().for_each(|(i, row)| {
			row.iter().enumerate().for_each(|(j, _)| {
				draw_rectangle(
					RECTSIZE_X*(i as f32),
					RECTSIZE_Y*(j as f32),
					RECTSIZE_X,
					RECTSIZE_Y,
					Color::from_vec(screen[i][j])
				);
			})
		});
		draw_text(&format!("VoxelTorus {}", (1.0 / get_frame_time()) as usize), 2.0, 16.0, 24.0, WHITE);
		next_frame().await;
	}
}
