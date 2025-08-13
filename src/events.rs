use raylib::prelude::*;
use raylib::consts::{GamepadAxis, GamepadButton};
use crate::{player::Player, maze::Maze};

const PLAYER_SPEED: f32 = 180.0;  // px/s
const ROT_SPEED: f32    = 2.2;    // rad/s (teclas)
const MOUSE_SENS: f32   = 0.0025; // rad/pixel
const GAMEPAD_DEADZONE: f32 = 0.20;

pub fn process_events(
    rl: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
    dt: f32,
    obstacles: &[(f32,f32,f32)], // <-- (x, y, r)
) {
    // Rotación teclas + mouse
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= ROT_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += ROT_SPEED * dt; }
    let md = rl.get_mouse_delta(); player.a += md.x * MOUSE_SENS;

    // Movimiento (teclas)
    let mut dx = 0.0; let mut dy = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_UP)    || rl.is_key_down(KeyboardKey::KEY_W) { dx += player.a.cos() * PLAYER_SPEED * dt; dy += player.a.sin() * PLAYER_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_DOWN)  || rl.is_key_down(KeyboardKey::KEY_S) { dx -= player.a.cos() * PLAYER_SPEED * dt; dy -= player.a.sin() * PLAYER_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_A) { dx += (player.a - std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
                                            dy += (player.a - std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_D) { dx += (player.a + std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
                                            dy += (player.a + std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt; }



    // Colisiones contra mapa (slide) + 
    let r = (block_size as f32) * 0.20;

    // mover en X
    let new_x = player.pos.x + dx;
    if can_move_maze(maze, block_size, new_x, player.pos.y, r)
        && !blocked_by_obstacles(new_x, player.pos.y, r, obstacles)
    {
        player.pos.x = new_x;
    }

    // mover en Y
    let new_y = player.pos.y + dy;
    if can_move_maze(maze, block_size, player.pos.x, new_y, r)
        && !blocked_by_obstacles(player.pos.x, new_y, r, obstacles)
    {
        player.pos.y = new_y;
    }
}

fn blocked_by_obstacles(x: f32, y: f32, r: f32, obs: &[(f32,f32,f32)]) -> bool {
    for &(ox, oy, orad) in obs {
        let dx = x - ox; let dy = y - oy;
        if dx*dx + dy*dy < (r + orad) * (r + orad) {
            return true;
        }
    }
    false
}

fn can_move_maze(maze: &Maze, block_size: usize, x: f32, y: f32, r: f32) -> bool {
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
