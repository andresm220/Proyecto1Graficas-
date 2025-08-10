// src/main.rs

mod framebuffer;
mod maze;
mod player;

mod renderer3d;
mod caster;
mod events;

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, make_maze, Maze};
use crate::player::Player;
use crate::renderer3d::render3d;
use crate::caster::cast_ray;
use crate::events::process_events;
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

    let mut framebuffer =
        Framebuffer::new(screen_w as u32, screen_h as u32);

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

    // 4) Bucle principal
    while !rl.window_should_close() {
        // 4.1 – manejo de input (rotar y mover jugador)
        process_events(&rl, &mut player);

        let mut d = rl.begin_drawing(&thread);
        framebuffer.clear(Color::BLACK);

       
        // 4.3 – render 3D estilo “Wolfenstein”
        render3d(&mut framebuffer, &maze, &player, block_size);

        // 4.4 – pinta todo en pantalla
        framebuffer.draw(&mut d);
        d.draw_text("←→ rotan, ↑↓ caminan", 10, 10, 20, Color::WHITE);
    }
}
