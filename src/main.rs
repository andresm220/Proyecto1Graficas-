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

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Title,
    Playing,
}

fn draw_title_screen(d: &mut raylib::drawing::RaylibDrawHandle, w: i32, h: i32) {
    use raylib::prelude::*;
    d.clear_background(Color::DARKBLUE);

    let title = "3D Maze por Andres Mazariegos";
    let subtitle = "  ¡Bienvenid@!";
    let hint1 = "ENTER / ESPACIO para empezar";
    let hint2 = "F11: Fullscreen   ESC: Salir";

    let title_size = 48;
    let text_size  = 22;

    // centrar textos
    let tw = d.measure_text(title, title_size);
    d.draw_text(title, (w - tw)/2, h/3 - 30, title_size, Color::WHITE);

    let sw = d.measure_text(subtitle, text_size);
    d.draw_text(subtitle, (w - sw)/2, h/3 + 30, text_size, Color::LIGHTGRAY);

    // pequeño parpadeo en el prompt
    let blink_on = (d.get_time() as i32 % 2) == 0;
    if blink_on {
        let hw = d.measure_text(hint1, text_size);
        d.draw_text(hint1, (w - hw)/2, (h*2)/3, text_size, Color::YELLOW);
    }

    let hw2 = d.measure_text(hint2, 18);
    d.draw_text(hint2, (w - hw2)/2, (h*2)/3 + 40, 18, Color::RAYWHITE);
}

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

    // Estado inicial: pantalla de bienvenida
    let mut state = GameState::Title; 

    // 4) Bucle principal
 while !rl.window_should_close() {
    // --- Inputs "globales" antes de dibujar ---
    if rl.is_key_pressed(KeyboardKey::KEY_F11) {
        rl.toggle_fullscreen();
    }

    // Resize framebuffer si cambió la ventana
    let cur_w = rl.get_screen_width();
    let cur_h = rl.get_screen_height();
    if cur_w != screen_w || cur_h != screen_h {
        screen_w = cur_w;
        screen_h = cur_h;
        framebuffer.resize(screen_w as u32, screen_h as u32);
    }

    // --- Lógica por estado (sin dibujar aún) ---
    match state {
        GameState::Title => {
            // Cambiar a juego con Enter o Espacio
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                state = GameState::Playing;
            }
        }
        GameState::Playing => {
            // Input + física SOLO en juego
            let dt = rl.get_frame_time();
            process_events(&rl, &mut player, &maze, block_size, dt);
        }
    }

    // --- Dibujo ---
    let mut d = rl.begin_drawing(&thread);

    match state {
        GameState::Title => {
            draw_title_screen(&mut d, screen_w, screen_h);
        }
        GameState::Playing => {
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
            d.draw_text("Izq/Der giran, Arr/Ab avanzan", 10, 40, 20, Color::WHITE);
        }
    }
}
}
