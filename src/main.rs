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
mod sprites; // sprites 2D en el mundo (billboard)

use minimap::{draw_minimap, MiniMapOpts, Corner};

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, make_maze, Maze};
use crate::player::Player;
use crate::renderer3d::render3d;
use crate::events::process_events;
use crate::texture::Texture;
use crate::textures::TextureAtlas;
use crate::sprites::{Sprite, SpriteAnim, render_sprites};

use raylib::prelude::*;
use std::env;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Title,
    Playing,
    Win,
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

    let tw = d.measure_text(title, title_size);
    d.draw_text(title, (w - tw)/2, h/3 - 30, title_size, Color::WHITE);

    let sw = d.measure_text(subtitle, text_size);
    d.draw_text(subtitle, (w - sw)/2, h/3 + 30, text_size, Color::LIGHTGRAY);

    let blink_on = (d.get_time() as i32 % 2) == 0;
    if blink_on {
        let hw = d.measure_text(hint1, text_size);
        d.draw_text(hint1, (w - hw)/2, (h*2)/3, text_size, Color::YELLOW);
    }

    let hw2 = d.measure_text(hint2, 18);
    d.draw_text(hint2, (w - hw2)/2, (h*2)/3 + 40, 18, Color::RAYWHITE);
}
//Pantalla de victoria
fn draw_win_screen(d: &mut raylib::drawing::RaylibDrawHandle, w: i32, h: i32, seconds: f32) {
    use raylib::prelude::*;
    d.clear_background(Color::DARKGREEN);

    let title = "¡Felicidades,laberinto completado!";
    let time  = format!("Tiempo: {:.2} s", seconds.max(0.0));
    let hint1 = "ENTER: Volver al menú";
    let hint2 = "R: Jugar de nuevo   F11: Fullscreen";

    let title_size = 44;
    let text_size  = 22;

    let tw = d.measure_text(title, title_size);
    d.draw_text(title, (w - tw)/2, h/3 - 20, title_size, Color::WHITE);

    let tw2 = d.measure_text(&time, text_size);
    d.draw_text(&time, (w - tw2)/2, h/3 + 30, text_size, Color::YELLOW);

    let hw1 = d.measure_text(hint1, 20);
    d.draw_text(hint1, (w - hw1)/2, (h*2)/3, 20, Color::RAYWHITE);
    let hw2 = d.measure_text(hint2, 18);
    d.draw_text(hint2, (w - hw2)/2, (h*2)/3 + 32, 18, Color::LIGHTGRAY);
}

fn place_player_at_start(player: &mut Player, maze: &Maze, block_size: usize) {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'p' {
                player.pos.x = (i * block_size + block_size / 2) as f32;
                player.pos.y = (j * block_size + block_size / 2) as f32;
                return;
            }
        }
    }
}

fn main() {
    // 1) Ventana y framebuffer
    let mut screen_w = 800;
    let mut screen_h = 600;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Maze Raycaster 3D")
        .build();

    rl.set_target_fps(15); // requisito ~15 FPS visible
    rl.hide_cursor();

    let mut framebuffer = Framebuffer::new(screen_w as u32, screen_h as u32);

    // z-buffer (para oclusión de sprites)
    let mut zbuffer: Vec<f32> = vec![f32::INFINITY; framebuffer.width as usize];

    // 2) Laberinto
    let args: Vec<String> = env::args().collect();
    let maze: Maze = if args.len() > 1 { load_maze(&args[1]) } else { make_maze(20, 15) };

    // Tamaño de celda del mundo
    let block_size: usize = 64;

    // 3) Player
    let mut player = Player::new(0.0, 0.0, std::f32::consts::PI / 4.0, std::f32::consts::PI / 3.0);
    place_player_at_start(&mut player, &maze, block_size);

    // 3.1) Texturas y atlas de paredes
    let brick_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("bricks.jpg");
    let brick = Rc::new(
        Texture::from_file(brick_path.to_str().unwrap())
            .expect("No se pudo cargar textura ladrillo"),
    );
    let mut atlas = TextureAtlas::new(brick.clone());
    atlas.insert('#', brick.clone());

    let stone = Rc::new(Texture::from_file(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("stone.jpg").to_str().unwrap()
    ).expect("No se pudo cargar textura piedra"));
    atlas.insert('A', stone);

    // === Sprite: CRATE pequeño con “temblor”, pegado a pared pero un poco separado ===
    let crate_tex = Rc::new(Texture::from_file(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("crate.png").to_str().unwrap()
    ).expect("No se pudo cargar assets/crate.png"));
    let crate_anim = SpriteAnim { frames: vec![crate_tex.clone()], fps: 1.0 };

    let mut sprites_world: Vec<Sprite> = Vec::new();
    let mut obstacles: Vec<(f32,f32,f32)> = Vec::new();

    let crate_radius = block_size as f32 * 0.10;  // colisión pequeña
    let side_offset  = block_size as f32 * 0.30;  // pegar a pared
    let center_nudge = block_size as f32 * 0.05;  // separar un poco al centro
    let is_wall = |ch: char| ch == '#' || ch == 'A';

    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c != ' ' { continue; }

            // pared adyacente
            let right_wall = i + 1 < row.len()  && is_wall(maze[j][i + 1]);
            let left_wall  = i >= 1             && is_wall(maze[j][i - 1]);
            let down_wall  = j + 1 < maze.len() && is_wall(maze[j + 1][i]);
            let up_wall    = j >= 1             && is_wall(maze[j - 1][i]);

            if !(right_wall || left_wall || up_wall || down_wall) { continue; }
            if ((i + j) % 10) != 0 { continue; } // espaciado

            // centro celda
            let mut cx = (i as f32 + 0.5) * block_size as f32;
            let mut cy = (j as f32 + 0.5) * block_size as f32;

            // pegar hacia la pared cercana + nudging al centro
            if      right_wall { cx += side_offset; cx -= center_nudge; }
            else if left_wall  { cx -= side_offset; cx += center_nudge; }
            else if down_wall  { cy += side_offset; cy -= center_nudge; }
            else if up_wall    { cy -= side_offset; cy += center_nudge; }

            // evitar spawn cerca del inicio
            let dx = cx - player.pos.x;
            let dy = cy - player.pos.y;
            if (dx*dx + dy*dy).sqrt() < (block_size as f32 * 2.0) { continue; }

            // desfase para el temblor
            let phase = ((i as f32) * 0.37 + (j as f32) * 0.61) % (2.0 * std::f32::consts::PI);

            sprites_world.push(Sprite {
                x: cx,
                y: cy,
                size: block_size as f32 * 0.40, 
                anim: crate_anim.clone(),
                wobble_amp: 1.5,
                wobble_freq: 2.2,
                phase,
            });
            obstacles.push((cx, cy, crate_radius));
        }
    }

    // tiempo de nivel y tiempo final mostrado en Win
    let mut level_time: f32 = 0.0;
    let mut win_time: Option<f32> = None;

    // Estado inicial
    let mut state = GameState::Title;

    // 4) Bucle principal
    while !rl.window_should_close() {
        // global input
        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        // resize framebuffer y zbuffer si cambió tamaño
        let cur_w = rl.get_screen_width();
        let cur_h = rl.get_screen_height();
        if cur_w != screen_w || cur_h != screen_h {
            screen_w = cur_w;
            screen_h = cur_h;
            framebuffer.resize(screen_w as u32, screen_h as u32);
            zbuffer.resize(framebuffer.width as usize, f32::INFINITY);
        }

        // --- Lógica por estado  ---
        match state {
            GameState::Title => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    // reset de tiempo y posición al iniciar
                    level_time = 0.0;
                    place_player_at_start(&mut player, &maze, block_size);
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                let dt = rl.get_frame_time();
                level_time += dt;
                process_events(&rl, &mut player, &maze, block_size, dt, &obstacles);

                // ¿llegó a la meta 'g'?
                let i = (player.pos.x as usize) / block_size;
                let j = (player.pos.y as usize) / block_size;
                if j < maze.len() && i < maze[0].len() && maze[j][i] == 'g' {
                    win_time = Some(level_time);
                    state = GameState::Win;
                }
            }
            GameState::Win => {
                // Volver al menú
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::Title;
                }
                // Reintentar: resetea tiempo y jugador, vuelve a Playing
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    level_time = 0.0;
                    win_time = None;
                    place_player_at_start(&mut player, &maze, block_size);
                    state = GameState::Playing;
                }
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

                zbuffer.fill(f32::INFINITY);
                render3d(&mut framebuffer, &maze, &player, block_size, &atlas, &mut zbuffer);

                render_sprites(&mut framebuffer, &player, &sprites_world, &zbuffer, player.fov, level_time);

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
            GameState::Win => {
                let t = win_time.unwrap_or(level_time);
                draw_win_screen(&mut d, screen_w, screen_h, t);
            }
        }
    }
}
