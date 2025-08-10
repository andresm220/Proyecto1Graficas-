// src/main.rs

mod framebuffer;
mod maze;
mod player;

mod renderer3d;
mod caster;
mod events;
mod texture;
mod textures;
mod minimap; 
use minimap::{draw_minimap, MiniMapOpts, Corner};

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, make_maze, Maze};
use crate::player::Player;
use crate::renderer3d::render3d;
use crate::events::process_events;
use crate::texture::Texture;
use crate::textures::TextureAtlas;
use raylib::prelude::*;
use std::env;
use std::path::Path;
use std::rc::Rc;

fn main() {
    // 1) Inicialización de la ventana y framebuffer
    let mut screen_w = 800;
    let mut screen_h = 600;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Maze Raycaster 3D")
        .build();

    rl.set_target_fps(15);   // para el requisito (~15 FPS)
    rl.hide_cursor();

    let mut framebuffer = Framebuffer::new(screen_w as u32, screen_h as u32);

    // 2) Carga o genera el laberinto
    let args: Vec<String> = env::args().collect();
    let mut maze: Maze = if args.len() > 1 {
        load_maze(&args[1])
    } else {
        make_maze(20, 15)
    };

    // Tamaño de celda del mundo 
    let block_size: usize = 64;

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

    // 3.1) Cargar texturas y atlas
    let brick_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("bricks.jpg");
    let brick = Rc::new(
        Texture::from_file(brick_path.to_str().unwrap())
            .expect("No se pudo cargar textura ladrillo"),
    );

    let mut atlas = TextureAtlas::new(brick.clone());
    atlas.insert('#', brick.clone()); // '#': muros estándar

    let stone = Rc::new(Texture::from_file(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("stone.jpg")
            .to_str()
            .unwrap(),
    ).expect("No se pudo cargar textura piedra"));
    atlas.insert('A', stone); // 'A': piedra 

    // 4) Bucle principal
    while !rl.window_should_close() {
        // Toggle fullscreen opcional (F11)
        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        // Detectar cambio de tamaño y redimensionar framebuffer
        let cur_w = rl.get_screen_width();
        let cur_h = rl.get_screen_height();
        if cur_w != screen_w || cur_h != screen_h {
            screen_w = cur_w;
            screen_h = cur_h;
            framebuffer.resize(screen_w as u32, screen_h as u32);
        }

        // Input + física
        let dt = rl.get_frame_time();
        process_events(&rl, &mut player, &maze, block_size, dt);

        // Render
        let mut d = rl.begin_drawing(&thread);
        framebuffer.clear(Color::BLACK);

        render3d(&mut framebuffer, &maze, &player, block_size, &atlas);

        framebuffer.draw(&mut d);
        draw_minimap(
            &mut d,
            &maze,
            &player,
            block_size,
            screen_w, screen_h,
    MiniMapOpts { tile: 6, margin: 10, corner: Corner::TopRight }
        );
        d.draw_fps(10, 10);
        d.draw_text("< >  girar, Ʌ V avanzar/retroceder ", 10, 40, 20, Color::WHITE);
    }
}
