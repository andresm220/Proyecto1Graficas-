// src/events.rs
use raylib::prelude::RaylibHandle;
use raylib::consts::KeyboardKey;
use crate::player::Player;
use std::f32::consts::PI;

/// Rota con ←→ y avanza/retrocede con ↑↓
pub fn process_events(rl: &RaylibHandle, player: &mut Player) {
    const MOVE_SPEED: f32 = 2.0;
    const ROT_SPEED:  f32 = PI / 60.0;

    if rl.is_key_down(KeyboardKey::KEY_LEFT)  {
        player.a -= ROT_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += ROT_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_UP)    {
        player.pos.x += MOVE_SPEED * player.a.cos();
        player.pos.y += MOVE_SPEED * player.a.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN)  {
        player.pos.x -= MOVE_SPEED * player.a.cos();
        player.pos.y -= MOVE_SPEED * player.a.sin();
    }
}
