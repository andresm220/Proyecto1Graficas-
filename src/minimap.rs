use raylib::prelude::*;
use crate::{maze::Maze, player::Player};

pub enum Corner { TopLeft, TopRight, BottomLeft, BottomRight }

pub struct MiniMapOpts {
    pub tile: i32,     // tamaño de cada celda del minimapa (px)
    pub margin: i32,   // margen desde el borde de la ventana (px)
    pub corner: Corner // esquina donde dibujar
}

pub fn draw_minimap(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    screen_w: i32,
    screen_h: i32,
    opts: MiniMapOpts,
) {
    let w_cells = maze[0].len() as i32;
    let h_cells = maze.len() as i32;
    let map_w = w_cells * opts.tile;
    let map_h = h_cells * opts.tile;

    // esquina
    let (x0, y0) = match opts.corner {
        Corner::TopLeft     => (opts.margin, opts.margin),
        Corner::TopRight    => (screen_w - opts.margin - map_w, opts.margin),
        Corner::BottomLeft  => (opts.margin, screen_h - opts.margin - map_h),
        Corner::BottomRight => (screen_w - opts.margin - map_w, screen_h - opts.margin - map_h),
    };

    // fondo semi-transparente
    d.draw_rectangle(x0 - 3, y0 - 3, map_w + 6, map_h + 6, Color::new(0, 0, 0, 160));

    // celdas
    for j in 0..h_cells {
        for i in 0..w_cells {
            let c = maze[j as usize][i as usize];
            let col = match c {
                '#' => Color::DARKGRAY,
                'A' => Color::GRAY,
                'g' => Color::GREEN,
                'p' => Color::SKYBLUE,
                ' ' => Color::new(255, 255, 255, 40),
                _   => Color::new(255, 255, 255, 80),
            };
            d.draw_rectangle(x0 + i * opts.tile, y0 + j * opts.tile, opts.tile, opts.tile, col);
        }
    }

    // jugador 
    let px = x0 as f32 + (player.pos.x / block_size as f32) * opts.tile as f32;
    let py = y0 as f32 + (player.pos.y / block_size as f32) * opts.tile as f32;
    d.draw_circle(px as i32, py as i32, (opts.tile as f32) * 0.35, Color::RED);

    // dirección del jugador
    let len = (opts.tile as f32) * 1.6;
    let fx = px + player.a.cos() * len;
    let fy = py + player.a.sin() * len;
    d.draw_line(px as i32, py as i32, fx as i32, fy as i32, Color::YELLOW);

    // borde
    d.draw_rectangle_lines(x0 - 3, y0 - 3, map_w + 6, map_h + 6, Color::WHITE);
}
