use raylib::prelude::*;
use crate::{player::Player, maze::Maze};

const PLAYER_SPEED: f32 = 180.0;  // px/s
const ROT_SPEED: f32    = 2.2;    // rad/s (teclas)
const MOUSE_SENS: f32   = 0.0025; // rad/pixel 

pub fn process_events(
    rl: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
    dt: f32,
) {
    // --- ROTACIÓN por teclas ---
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= ROT_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += ROT_SPEED * dt; }

    // --- ROTACIÓN por mouse (solo horizontal) ---
    let md = rl.get_mouse_delta();   // Vector2: movimiento desde el frame anterior
    player.a += md.x * MOUSE_SENS;

    // --- MOVIMIENTO ---
    let mut dx = 0.0;
    let mut dy = 0.0;

    if rl.is_key_down(KeyboardKey::KEY_UP) {
        dx += player.a.cos() * PLAYER_SPEED * dt;
        dy += player.a.sin() * PLAYER_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        dx -= player.a.cos() * PLAYER_SPEED * dt;
        dy -= player.a.sin() * PLAYER_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        dx += (player.a - std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
        dy += (player.a - std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        dx += (player.a + std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
        dy += (player.a + std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt;
    }

    // Soporte de gamepad (id 0)
if rl.is_gamepad_available(0) {
    // sticks
    let lx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
    let ly = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
    let rx = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);

    // avanzar/retroceder con stick izquierdo (Y invertida)
    let fwd = -ly;
    let strafe = lx;

    // movimiento en mundo
    // adelante/atrás
    let speed = PLAYER_SPEED * dt;
    dx += player.a.cos() * fwd * speed;
    dy += player.a.sin() * fwd * speed;

    // strafe
    dx += (player.a + std::f32::consts::FRAC_PI_2).cos() * strafe * speed;
    dy += (player.a + std::f32::consts::FRAC_PI_2).sin() * strafe * speed;

    // rotación con stick derecho X
    player.a += rx * 2.5 * dt;

    // botones opcionales: D-Pad como teclas
    if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
        dx += player.a.cos() * speed;
        dy += player.a.sin() * speed;
    }
    if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
        dx -= player.a.cos() * speed;
        dy -= player.a.sin() * speed;
    }
}


    // --- COLISIONES (círculo que hace “slide”) ---
    let r = (block_size as f32) * 0.20;

    let new_x = player.pos.x + dx;
    if can_move_circle(maze, block_size, new_x, player.pos.y, r) {
        player.pos.x = new_x;
    }
    let new_y = player.pos.y + dy;
    if can_move_circle(maze, block_size, player.pos.x, new_y, r) {
        player.pos.y = new_y;
    }
}

fn can_move_circle(maze: &Maze, block_size: usize, x: f32, y: f32, r: f32) -> bool {
    let pts = [(x - r, y), (x + r, y), (x, y - r), (x, y + r)];
    pts.iter().all(|&(px, py)| is_walkable(maze, block_size, px, py))
}

fn is_walkable(maze: &Maze, block_size: usize, x: f32, y: f32) -> bool {
    if x < 0.0 || y < 0.0 { return false; }
    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;
    if j >= maze.len() || i >= maze[0].len() { return false; }
    let c = maze[j][i];
    c == ' ' || c == 'p' || c == 'g'
}
