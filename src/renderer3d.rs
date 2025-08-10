// src/renderer3d.rs

use raylib::color::Color;
use crate::{
    framebuffer::Framebuffer,
    maze::Maze,
    player::Player,
    caster::{cast_ray, Intersect},
};

pub fn render3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let num_rays        = framebuffer.width;
    let hw              = framebuffer.width  as f32 / 2.0;
    let hh              = framebuffer.height as f32 / 2.0;
    let dist_proj_plane = hw / (player.fov / 2.0).tan();

    // 1) Pintar cielo (mitad superior)
    framebuffer.set_current_color(Color::SKYBLUE);
    for y in 0..(hh as u32) {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    // 2) Pintar suelo (mitad inferior)
    framebuffer.set_current_color(Color::BLACK);
    for y in (hh as u32)..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    // Máxima distancia para sombreado (ancho del laberinto en “unidades de bloque”)
    let max_distance = (maze[0].len() * block_size) as f32;
    // Anchura de cada stake en píxeles
    let step = 2;

    // 3) Por cada “columna” de píxeles, lanzamos un rayo
    for col in (0..num_rays).step_by(step as usize) {
        let current_ray = col as f32 / num_rays as f32;
        let angle = player.a - (player.fov / 2.0)
                  + (player.fov * current_ray);

        // Intersect contiene distancia al muro y qué muro (char)
        let Intersect { distance, impact } =
            cast_ray(framebuffer, maze, player, angle, block_size, false);

        // Calcula la altura de la stake
        let stake_height = (block_size as f32 / distance) * dist_proj_plane;
        let top    = (hh - stake_height / 2.0).max(0.0) as usize;
        let bottom = (hh + stake_height / 2.0)
                    .min(framebuffer.height as f32) as usize;

        // Color base según tipo de muro
        let base = match impact {
            '#' => Color::GRAY,
            _   => Color::DARKGRAY,
        };
        // Factor de sombreado (1.0 cerca → 0.0 lejos)
        let shade = 1.0 - (distance / max_distance).min(1.0);
        let col_r = (base.r as f32 * shade) as u8;
        let col_g = (base.g as f32 * shade) as u8;
        let col_b = (base.b as f32 * shade) as u8;
        let shaded = Color::new(col_r, col_g, col_b, 255);
        framebuffer.set_current_color(shaded);

        // Dibuja la stake de ancho `step`
        for dx in 0..step {
            let x = (col + dx as u32).min(framebuffer.width - 1);
            for y in top..bottom {
                framebuffer.set_pixel(x, y as u32);
            }
        }
    }
}
