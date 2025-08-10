// src/main.rs

mod framebuffer;
mod maze;
mod player;

mod renderer3d;
mod caster;
mod events;
mod texture; // <-- NUEVO

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, make_maze, Maze};
use crate::player::Player;
use crate::renderer3d::render3d;
use crate::events::process_events;
use crate::texture::Texture; // <-- NUEVO
use raylib::prelude::*;
use std::env;

fn main() {
    // 1) Inicialización de la ventana y framebuffer
    let screen_w = 800;
    let screen_h = 600;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Maze Raycaster 3D")
        .build();

    let mut framebuffer = Framebuffer::new(screen_w as u32, screen_h as u32);

    // 2) Carga o genera el laberinto
    let args: Vec<String> = env::args().collect();
    let maze: Maze = if args.len() > 1 {
        load_maze(&args[1])
    } else {
        make_maze(20, 15)
    };
    let block_size = screen_h as usize / maze.len();

    // 3) Inicializa al jugador en la 'p' del mapa, con FOV de 60°
    let mut player = {
        let mut p = Player::new(
            0.0,
            0.0,
            std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 3.0,
        );
        for (j, row) in maze.iter().enumerate() {
            for (i, &c) in row.iter().enumerate() {
                if c == 'p' {
                    p.pos.x = (i * block_size + block_size / 2) as f32;
                    p.pos.y = (j * block_size + block_size / 2) as f32;
                }
            }
        }
        p
    };

    // 3.1) Cargar textura de pared desde assets/  
   
    let tex_path = format!(
        "{}/assets/bricks.jpg",
        env!("CARGO_MANIFEST_DIR")
    );
    let wall_tex = Texture::from_file(&tex_path)
        .expect("No se pudo cargar la textura de pared (JPG/PNG) en assets/");

    // 4) Bucle principal
   while !rl.window_should_close() {
    let dt = rl.get_frame_time(); // segundos desde el frame anterior

    // ahora pasamos maze, block_size y dt
    process_events(&rl, &mut player, &maze, block_size, dt);

    let mut d = rl.begin_drawing(&thread);
    framebuffer.clear(Color::BLACK);

    // render 3D (con textura)
    render3d(&mut framebuffer, &maze, &player, block_size, &wall_tex);

    // FPS visible (requisito del proyecto)
   

    framebuffer.draw(&mut d);
    let fps = d.get_fps();
    d.draw_text(&format!("FPS: {}", fps), screen_w - 90, 10, 20, Color::YELLOW);
    d.draw_text("←→ rotan, ↑↓ caminan", 10, 10, 20, Color::WHITE);
}
}
