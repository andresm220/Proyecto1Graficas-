// src/events.rs
use raylib::prelude::RaylibHandle;
use raylib::consts::KeyboardKey;
use crate::player::Player;
use std::f32::consts::PI;

use crate::maze::Maze;

const PLAYER_SPEED: f32 = 180.0; // px/s (ajusta a gusto)
const ROT_SPEED: f32    = 2.2;   // rad/s (ajusta a gusto)

pub fn process_events(
    rl: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
    dt: f32,
) {
    // ROTACIÓN (izq/der)
    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= ROT_SPEED * dt; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += ROT_SPEED * dt; }

    // DESEADA (dx, dy) según teclas
    let mut dx = 0.0;
    let mut dy = 0.0;

    // adelante/atrás
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        dx += player.a.cos() * PLAYER_SPEED * dt;
        dy += player.a.sin() * PLAYER_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        dx -= player.a.cos() * PLAYER_SPEED * dt;
        dy -= player.a.sin() * PLAYER_SPEED * dt;
    }

    // strafe (opcional)
    if rl.is_key_down(KeyboardKey::KEY_A) {
        dx += (player.a - std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
        dy += (player.a - std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        dx += (player.a + std::f32::consts::FRAC_PI_2).cos() * PLAYER_SPEED * dt;
        dy += (player.a + std::f32::consts::FRAC_PI_2).sin() * PLAYER_SPEED * dt;
    }

    // --- COLISIONES ---
    // Radio del jugador (~20% del bloque)
    let r = (block_size as f32) * 0.20;

    // intenta mover en X
    let new_x = player.pos.x + dx;
    if can_move_circle(maze, block_size, new_x, player.pos.y, r) {
        player.pos.x = new_x;
    }

    // intenta mover en Y
    let new_y = player.pos.y + dy;
    if can_move_circle(maze, block_size, player.pos.x, new_y, r) {
        player.pos.y = new_y;
    }
}

fn can_move_circle(maze: &Maze, block_size: usize, x: f32, y: f32, r: f32) -> bool {
    // chequea 4 puntos alrededor (colisión con AABB de pared)
    let pts = [
        (x - r, y),
        (x + r, y),
        (x, y - r),
        (x, y + r),
    ];
    pts.iter().all(|&(px, py)| is_walkable(maze, block_size, px, py))
}

fn is_walkable(maze: &Maze, block_size: usize, x: f32, y: f32) -> bool {
    if x < 0.0 || y < 0.0 { return false; }
    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;
    if j >= maze.len() || i >= maze[0].len() { return false; }
    let c = maze[j][i];
    // Define qué es caminado: espacio, inicio 'p' y meta 'g'
    c == ' ' || c == 'p' || c == 'g'
}
