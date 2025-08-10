// src/caster.rs

use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

/// Resultado del cast: distancia al muro y qué tipo de pared impactó.
pub struct Intersect {
    pub distance: f32,
    pub impact:   char,
}

/// Lanza un rayo desde la posición del jugador en ángulo `a`.
/// Si `draw_line == true`, además traza la línea en el minimapa.
pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0;
    framebuffer.set_current_color(Color::WHITESMOKE);

    loop {
        // Avanzamos un “paso” d en la dirección a
        let dx = d * a.cos();
        let dy = d * a.sin();
        let x = (player.pos.x + dx) as usize;
        let y = (player.pos.y + dy) as usize;

        // Índices de celda
        let i = x / block_size;
        let j = y / block_size;

        // Si salimos del mapa, devolvemos distancia actual
        if j >= maze.len() || i >= maze[0].len() {
            return Intersect { distance: d, impact: ' ' };
        }

        let cell = maze[j][i];
        // Si choca contra algo que no sea espacio/p/g
        if cell != ' ' && cell != 'p' && cell != 'g' {
            return Intersect { distance: d, impact: cell };
        }

        // Dibuja la línea en el minimapa 2D
        if draw_line {
            framebuffer.set_pixel(x as u32, y as u32);
        }

        // Incrementa distancia
        d += 10.0;
    }
}
