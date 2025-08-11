// src/renderer3d.rs
use raylib::color::Color;
use crate::{
    framebuffer::Framebuffer,
    maze::Maze,
    player::Player,
    caster::cast_ray,
    texture::Texture,
    textures::TextureAtlas,
};

/// Render de paredes + escritura de z-buffer (distancia perpendicular por columna).

pub fn render3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    atlas: &TextureAtlas,
    zbuf: &mut [f32],         
) {
    let num_rays        = framebuffer.width;
    let hw              = framebuffer.width  as f32 / 2.0;
    let hh              = framebuffer.height as f32 / 2.0;
    let dist_proj_plane = hw / (player.fov / 2.0).tan();

    // Cielo
    framebuffer.set_current_color(Color::SKYBLUE);
    for y in 0..(hh as u32) {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }
    // Suelo
    framebuffer.set_current_color(Color::BLACK);
    for y in (hh as u32)..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.set_pixel(x, y);
        }
    }

    let max_distance = (maze[0].len() * block_size) as f32;
    let step = 2;

    for col in (0..num_rays).step_by(step as usize) {
        let current_ray = col as f32 / num_rays as f32;
        let angle = player.a - (player.fov / 2.0) + (player.fov * current_ray);

        let hit = cast_ray(framebuffer, maze, player, angle, block_size, false);
        let distance  = hit.distance.max(0.0001);
        let dist_perp = distance * (angle - player.a).cos(); 

        // >>> Escribir z-buffer para estas columnas 
        for dx in 0..step {
            let x = (col + dx as u32).min(num_rays - 1) as usize;
            zbuf[x] = dist_perp;
        }

        // Altura de la columna
        let stake_height = (block_size as f32 / dist_perp) * dist_proj_plane;
        let top    = (hh - stake_height / 2.0).max(0.0) as i32;
        let bottom = (hh + stake_height / 2.0).min(framebuffer.height as f32) as i32;

        // Punto de impacto en mundo
        let hit_x = player.pos.x + distance * angle.cos();
        let hit_y = player.pos.y + distance * angle.sin();

        // Coordenadas dentro del bloque
        let lx = ((hit_x as i32 % block_size as i32) + block_size as i32) % block_size as i32;
        let ly = ((hit_y as i32 % block_size as i32) + block_size as i32) % block_size as i32;

        // Detectar lado -> coord U
        let to_left   = lx;
        let to_right  = (block_size as i32 - 1) - lx;
        let to_top    = ly;
        let to_bottom = (block_size as i32 - 1) - ly;

        let (_, mut u) = {
            let min_vert = to_left.min(to_right);
            let min_hori = to_top.min(to_bottom);
            if min_vert <= min_hori {
                (true,  ly as f32 / (block_size.saturating_sub(1)) as f32)
            } else {
                (false, lx as f32 / (block_size.saturating_sub(1)) as f32)
            }
        };
        if u < 0.0 { u += 1.0; }

        // Textura según el char del muro impactado
        let wall_tex: &Texture = atlas.get(hit.impact);

        // Sombreado por distancia
        let shade = 1.0 - (distance / max_distance).min(1.0);

        // Dibujar columna texturizada
        let y0 = top.max(0);
        let y1 = bottom.min(framebuffer.height as i32 - 1);
        let denom = (y1 - y0).max(1) as f32;

        for dx in 0..step {
            let x_screen = (col + dx as u32).min(framebuffer.width - 1) as i32;

            for y in y0..=y1 {
                let v = (y - y0) as f32 / denom; // 0..1 vertical
                let texel = wall_tex.sample(u, v);

                let a = ((texel >> 24) & 0xFF) as u8;
                let mut r = ((texel >> 16) & 0xFF) as f32;
                let mut g = ((texel >> 8)  & 0xFF) as f32;
                let mut b = ( texel        & 0xFF) as f32;
                r *= shade; g *= shade; b *= shade;

                framebuffer.set_current_color(Color::new(r as u8, g as u8, b as u8, a));
                framebuffer.set_pixel(x_screen as u32, y as u32);
            }
        }
    }
}
