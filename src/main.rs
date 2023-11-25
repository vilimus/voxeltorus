
// voxel raycasting in rust

use macroquad::prelude::*;
use nalgebra::Vector3;
use std::f32::consts::PI;

// universal constants
const WORLDSIZE: [usize; 3] = [16, 64, 16];
const LX: usize = WORLDSIZE[0];
const LY: usize = WORLDSIZE[1];
const LZ: usize = WORLDSIZE[2];

const SCREEN_SIZE: (f32, f32) = (640.0, 480.0);
const SCREEN: (usize, usize) = (64, 48);

const VIEW_DISTANCE: usize = 32;
const BREAK_DISTANCE: usize = 4;
const FOV: (f32, f32) = (PI/2.0, PI/2.0*(SCREEN.1 as f32)/(SCREEN.0 as f32));

const MOVEMENT_SPEED: f32 = 0.5;
const ROT_SPEED: (f32, f32) = (0.5, 0.5);
const AMBIENT: [f32; 3] = [0.75, 0.75, 1.0];

const RECT_SIZE_X: f32 = SCREEN_SIZE.0 / (SCREEN.0 as f32);
const RECT_SIZE_Y: f32 = SCREEN_SIZE.1 / (SCREEN.1 as f32);

#[derive(Copy, Clone)]
struct Cell {
    visible: bool,
    color: Vector3<f32>
}


fn load_world() -> Vec<Vec<Vec<Cell>>> {

    let air   = Cell {visible: false,  color: Vector3::new(0.0, 0.0, 0.0)};
    let stone = Cell {visible: true,   color: Vector3::new(0.1, 0.1, 0.1)};
    let dirt  = Cell {visible: true,   color: Vector3::new(0.2, 0.1, 0.0)};
    let grass = Cell {visible: true,   color: Vector3::new(0.0, 0.5, 0.0)};
    let sky   = Cell {visible: true,   color: Vector3::new(0.5, 0.5, 1.0)};
    let wood  = Cell {visible: true,   color: Vector3::new(0.5, 0.25, 0.0)};
    let leaves = Cell {visible: true,   color: Vector3::new(0.0, 0.25, 0.0)};

    let mut world: Vec<Vec<Vec<Cell>>> = vec![vec![vec![air; LZ]; LY]; LX];

    for i in 0..LX {
        for k in 0..LZ {
            for j in 0..44 {
                world[i][j][k] = stone;
            }
            for j in 44..47 {
                world[i][j][k] = dirt;
            }
            world[i][47][k] = grass;
            world[i][63][k] = sky;
        }
    }
    for i in 22..11 {
        for j in 6..11 {
            world[i][50][j] = leaves;
            world[i][51][j] = leaves;
        }
    }
    for i in 7..10 {
        for j in 7..10 {
            world[i][52][j] = leaves;
            world[i][53][j] = leaves;
        }
    }
    world[8][54][8] = leaves;
    for i in 48..54 {
        world[8][i][8] = wood;
    }
    return world;
}

fn lattice_intersect(pos: Vector3<f32>, v: Vector3<f32>) -> (Vector3<f32>, Vector3<i32>) {
    let t = Vector3::new(
        ((v.x.signum() + 1.0) / 2.0 - pos.x) / v.x,
        ((v.y.signum() + 1.0) / 2.0 - pos.y) / v.y,
        ((v.z.signum() + 1.0) / 2.0 - pos.z) / v.z,
    );
    let mut i_min: usize = 0;
    let t_min: f32 = t.min();
    for i in 0..3 {
        if t[i] == t_min {
            i_min = i
        }
    }
    let t_star = t[i_min];
    let mut key = Vector3::new(0, 0, 0);
    key[i_min] = v[i_min].signum() as i32;
    let key2 = key.map(|x| x as f32);
    let x_new = pos + t_star*v - key2;
    return (x_new, key);
}

fn raycast(world: &Vec<Vec<Vec<Cell>>>, cell_x0: Vector3<usize>, x0: Vector3<f32>, v: Vector3<f32>, max_steps: usize) -> (Vector3<usize>, usize) {
    let mut cell_x = cell_x0;
    let mut x = x0;
    let mut steps = 0;
    let mut key = Vector3::new(0, 0, 0);
    for _ in 0..max_steps {
        (x, key) = lattice_intersect(x, v);
        cell_x.x = ((cell_x.x as i32) + key.x).rem_euclid(LX as i32) as usize;
        cell_x.y = ((cell_x.y as i32) + key.y).rem_euclid(LY as i32) as usize;
        cell_x.z = ((cell_x.z as i32) + key.z).rem_euclid(LZ as i32) as usize;
        if world[cell_x.x][cell_x.y][cell_x.z].visible == true {
            break;
        }
        steps = steps + 1;
    }
    return (cell_x, steps)
}

#[macroquad::main("voxeltorusrs")]
async fn main() {
    request_new_screen_size(SCREEN_SIZE.0, SCREEN_SIZE.1);

    let mut world = load_world();

    let mut cell: Vector3<usize> = Vector3::new(0, 48, 0);
    let mut pos: Vector3<f32> = Vector3::new(0.5, 0.5, 0.5); // these two should essentially be in a pair..

    let mut cam: [f32; 2] = [0.0, 0.0]; // camera angles, horizontal/vertical rotation

    let mut look : Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
    let mut up   : Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let mut right: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

    set_cursor_grab(true);
    show_mouse(false);

    let rect_size_x = SCREEN_SIZE.0 / (SCREEN.0 as f32);
    let rect_size_y = SCREEN_SIZE.1 / (SCREEN.1 as f32);

    let mut alive = true;
	let mut fps = 0.0;
    while alive {
        if is_key_pressed(KeyCode::Escape) {
            alive = false;
        }

        let mut vel = Vector3::new(0.0, 0.0, 0.0);
        if is_key_down(KeyCode::E) {
            vel.y = vel.y + 1.0
        }
        if is_key_down(KeyCode::Q) {
            vel.y = vel.y - 1.0
        }
        if is_key_down(KeyCode::W) {
            vel = vel + look;
        }
        if is_key_down(KeyCode::S) {
            vel = vel - look;
        }
        if is_key_down(KeyCode::A) {
            vel = vel - right;
        }
        if is_key_down(KeyCode::D) {
            vel = vel + right;
        }
		if is_key_down(KeyCode::I) {
			println!("fps: {}", fps);
        }
        let vel_norm = vel.norm();
        if vel_norm > 0.0 {
            pos = pos + MOVEMENT_SPEED * vel / vel.norm();
        }
        if pos.x < 0.0 {
            cell.x = (((cell.x as i32) - 1).rem_euclid(LX as i32)) as usize;
            pos.x = pos.x + 1.0;
        }
        if pos.y < 0.0 {
            cell.y = (((cell.y as i32) - 1).rem_euclid(LY as i32)) as usize;
            pos.y = pos.y + 1.0;
        }
        if pos.z < 0.0 {
            cell.z = (((cell.z as i32) - 1).rem_euclid(LZ as i32)) as usize;
            pos.z = pos.z + 1.0;
        }
        if pos.x > 1.0 {
            cell.x = (((cell.x as i32) + 1).rem_euclid(LX as i32)) as usize;
            pos.x = pos.x - 1.0;
        }
        if pos.y > 1.0 {
            cell.y = (((cell.y as i32) + 1).rem_euclid(LY as i32)) as usize;
            pos.y = pos.y - 1.0;
        }
        if pos.z > 1.0 {
            cell.z = (((cell.z as i32) + 1).rem_euclid(LZ as i32)) as usize;
            pos.z = pos.z - 1.0;
        }

        if is_key_down(KeyCode::Left) {
            cam[0] = cam[0] - ROT_SPEED.0;
        }
        if is_key_down(KeyCode::Right) {
            cam[0] = cam[0] + ROT_SPEED.0;
        }
        if is_key_down(KeyCode::Up) {
            cam[1] = cam[1] + ROT_SPEED.1;
        }
        if is_key_down(KeyCode::Down) {
            cam[1] = cam[1] - ROT_SPEED.1;
        }

        let mouse_delta = mouse_delta_position();
        cam[0] = cam[0] - ROT_SPEED.0 * mouse_delta.x;
        cam[1] = cam[1] + ROT_SPEED.1 * mouse_delta.y;

        look  = Vector3::new( cam[0].cos()*cam[1].cos(), cam[1].sin(),  cam[0].sin()*cam[1].cos());
        up    = Vector3::new(-cam[0].cos()*cam[1].sin(), cam[1].cos(), -cam[0].sin()*cam[1].sin());
        right = Vector3::new(-cam[0].sin(),              0.0,           cam[0].cos());

        if is_mouse_button_pressed(MouseButton::Left) {
            let (cell_x, steps) = raycast(&world, cell, pos, look, BREAK_DISTANCE);
            world[cell_x.x][cell_x.y][cell_x.z].visible = false;
        }
        for i in 0..(SCREEN.0) {
            for j in 0..(SCREEN.1) {
                let a_i = (((i as f32) / (SCREEN.0 as f32) - 0.5) * FOV.0).atan();
                let b_j = (((j as f32) / (SCREEN.1 as f32) - 0.5) * FOV.1).atan();
                let look_ij = look + a_i*right - b_j*up;

                let (cell_x, steps) = raycast(&world, cell, pos, look_ij, VIEW_DISTANCE);

                let cell_hit = world[cell_x.x][cell_x.y][cell_x.z];
                let fade: f32 = 1.0 - (steps as f32) / (VIEW_DISTANCE as f32);

                let color = Color::new(
                    cell_hit.color.x * fade + AMBIENT[0] * (1.0 - fade),
                    cell_hit.color.y * fade + AMBIENT[1] * (1.0 - fade),
                    cell_hit.color.z * fade + AMBIENT[2] * (1.0 - fade),
                    1.0
                );
                draw_rectangle(RECT_SIZE_X*(i as f32), RECT_SIZE_Y*(j as f32), RECT_SIZE_X, RECT_SIZE_Y, color)
            }
        }
        next_frame().await;
        fps = 1.0/get_frame_time();
        //println!("FPS: {}", fps);
    }
}
