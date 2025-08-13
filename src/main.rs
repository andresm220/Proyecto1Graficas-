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

#[derive(Clone, Copy)]
struct LevelConfig {
    name: &'static str,
    cells: (usize, usize), // (cell_w, cell_h)
}

// Tres laberintos más pequeños 
const LEVELS: [LevelConfig; 3] = [
    LevelConfig { name: "Nivel 1 (12x9)",  cells: (12, 9)  },
    LevelConfig { name: "Nivel 2 (14x10)", cells: (14, 10) },
    LevelConfig { name: "Nivel 3 (16x12)", cells: (16, 12) },
];

fn draw_title_screen(d: &mut raylib::drawing::RaylibDrawHandle, w: i32, h: i32, sel: usize) {
    use raylib::prelude::*;
    d.clear_background(Color::DARKBLUE);

    let title   = "3D Maze por Andres Mazariegos";
    let subtitle= "Selecciona nivel y presiona ENTER";
    let hint2   = "F11: Fullscreen   ESC: Salir   (1/2/3 o flechas para elegir)";

    let title_size = 44;
    let text_size  = 22;

    let tw = d.measure_text(title, title_size);
    d.draw_text(title, (w - tw)/2, h/6, title_size, Color::WHITE);

    let sw = d.measure_text(subtitle, text_size);
    d.draw_text(subtitle, (w - sw)/2, h/6 + 50, text_size, Color::LIGHTGRAY);

    // opciones de nivel
    let base_y = h/3;
    for (i, lv) in LEVELS.iter().enumerate() {
        let txt = lv.name;
        let col = if i == sel { Color::YELLOW } else { Color::RAYWHITE };
        let sz  = if i == sel { 26 } else { 22 };
        let twx = d.measure_text(txt, sz);
        d.draw_text(txt, (w - twx)/2, base_y + (i as i32)*40, sz, col);
    }

    let hw2 = d.measure_text(hint2, 18);
    d.draw_text(hint2, (w - hw2)/2, (h*5)/6, 18, Color::LIGHTGRAY);
}

fn draw_win_screen(d: &mut raylib::drawing::RaylibDrawHandle, w: i32, h: i32, seconds: f32) {
    use raylib::prelude::*;
    d.clear_background(Color::DARKGREEN);

    let title = "¡Nivel completado!";
    let time  = format!("Tiempo: {:.2} s", seconds.max(0.0));
    let hint1 = "ENTER: Volver al menú";
    let hint2 = "R: Reintentar el mismo nivel   F11: Fullscreen";

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

fn find_start(maze: &Maze, block_size: usize) -> (f32, f32) {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'p' {
                return ((i * block_size + block_size / 2) as f32,
                        (j * block_size + block_size / 2) as f32);
            }
        }
    }
    (block_size as f32 * 0.5, block_size as f32 * 0.5) // fallback
}

/// Construye un nivel: maze + posiciones de sprites (crates pegadas a pared) + obstáculos
fn build_level(cfg: LevelConfig, block_size: usize) -> (Maze, Vec<Sprite>, Vec<(f32,f32,f32)>) {
    let maze = make_maze(cfg.cells.0, cfg.cells.1);

    let (px, py) = find_start(&maze, block_size);

    let mut sprites_world: Vec<Sprite> = Vec::new();
    let mut obstacles: Vec<(f32,f32,f32)> = Vec::new();

    let crate_radius = block_size as f32 * 0.10;
    let side_offset  = block_size as f32 * 0.30;
    let center_nudge = block_size as f32 * 0.05;
    let is_wall = |ch: char| ch == '#' || ch == 'A';

    #[derive(Clone, Copy)]
    struct Pos { cx: f32, cy: f32, phase: f32 }
    let mut poslist: Vec<Pos> = Vec::new();

    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c != ' ' { continue; }
            let right_wall = i + 1 < row.len()  && is_wall(maze[j][i + 1]);
            let left_wall  = i >= 1             && is_wall(maze[j][i - 1]);
            let down_wall  = j + 1 < maze.len() && is_wall(maze[j + 1][i]);
            let up_wall    = j >= 1             && is_wall(maze[j - 1][i]);
            if !(right_wall || left_wall || up_wall || down_wall) { continue; }
            if ((i + j) % 10) != 0 { continue; }

            let mut cx = (i as f32 + 0.5) * block_size as f32;
            let mut cy = (j as f32 + 0.5) * block_size as f32;

            if      right_wall { cx += side_offset; cx -= center_nudge; }
            else if left_wall  { cx -= side_offset; cx += center_nudge; }
            else if down_wall  { cy += side_offset; cy -= center_nudge; }
            else if up_wall    { cy -= side_offset; cy += center_nudge; }

            let dx = cx - px;
            let dy = cy - py;
            if (dx*dx + dy*dy).sqrt() < (block_size as f32 * 2.0) { continue; }

            let phase = ((i as f32) * 0.37 + (j as f32) * 0.61) % (2.0 * std::f32::consts::PI);
            poslist.push(Pos { cx, cy, phase });
            obstacles.push((cx, cy, crate_radius));
        }
    }

    for p in poslist {
        sprites_world.push(Sprite {
            x: p.cx,
            y: p.cy,
            size: block_size as f32 * 0.40,
            anim: SpriteAnim { frames: Vec::new(), fps: 1.0 }, // se rellena en main
            wobble_amp: 1.5,
            wobble_freq: 2.2,
            phase: p.phase,
        });
    }

    (maze, sprites_world, obstacles)
}

fn main() {
    // 1) Ventana y framebuffer
    let mut screen_w = 800;
    let mut screen_h = 600;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Maze Raycaster 3D")
        .build();

   
    rl.hide_cursor();

    let mut framebuffer = Framebuffer::new(screen_w as u32, screen_h as u32);
    let mut zbuffer: Vec<f32> = vec![f32::INFINITY; framebuffer.width as usize];

    // 2) Texturas y atlas de paredes (una vez)
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

    // Sprite crate (anim 1 frame, se comparte entre niveles)
    let crate_tex = Rc::new(Texture::from_file(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("crate.png").to_str().unwrap()
    ).expect("No se pudo cargar assets/crate.png"));
    let crate_anim = SpriteAnim { frames: vec![crate_tex.clone()], fps: 1.0 };

    // Parámetros de mundo
    let block_size: usize = 64;

    // === Inicialización por defecto para evitar E0381 ===
    let default_cfg = LEVELS[0];
    let mut maze = make_maze(default_cfg.cells.0, default_cfg.cells.1);
    let mut sprites_world: Vec<Sprite> = Vec::new();
    let mut obstacles: Vec<(f32,f32,f32)> = Vec::new();

    // Player y tiempos
    let mut player = Player::new(0.0, 0.0, std::f32::consts::PI / 4.0, std::f32::consts::PI / 3.0);
    place_player_at_start(&mut player, &maze, block_size);
    let mut level_time: f32 = 0.0;
    let mut win_time: Option<f32> = None;

    // Selección de nivel en el menú
    let mut selected_level: usize = 0;

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

        // --- Lógica por estado (sin dibujar aún) ---
        match state {
            GameState::Title => {
                // cambiar selección (flechas o 1/2/3)
                if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                    if selected_level == 0 { selected_level = LEVELS.len() - 1; }
                    else { selected_level -= 1; }
                }
                if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                    selected_level = (selected_level + 1) % LEVELS.len();
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ONE)   { selected_level = 0; }
                if rl.is_key_pressed(KeyboardKey::KEY_TWO)   { selected_level = 1.min(LEVELS.len()-1); }
                if rl.is_key_pressed(KeyboardKey::KEY_THREE) { selected_level = 2.min(LEVELS.len()-1); }

                // Iniciar nivel seleccionado
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    let (mz, mut spr, obs) = build_level(LEVELS[selected_level], block_size);
                    for s in spr.iter_mut() { s.anim = crate_anim.clone(); }
                    maze = mz;
                    sprites_world = spr;
                    obstacles = obs;

                    place_player_at_start(&mut player, &maze, block_size);
                    level_time = 0.0;
                    win_time = None;
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                let dt = rl.get_frame_time();
                level_time += dt;
                // eventos + colisiones (mapa + obstáculos)
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
                // Reintentar el MISMO nivel
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    let (mz, mut spr, obs) = build_level(LEVELS[selected_level], block_size);
                    for s in spr.iter_mut() { s.anim = crate_anim.clone(); }
                    maze = mz;
                    sprites_world = spr;
                    obstacles = obs;

                    place_player_at_start(&mut player, &maze, block_size);
                    level_time = 0.0;
                    win_time = None;
                    state = GameState::Playing;
                }
            }
        }

        // --- Dibujo ---
        let mut d = rl.begin_drawing(&thread);

        match state {
            GameState::Title => {
                draw_title_screen(&mut d, screen_w, screen_h, selected_level);
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
