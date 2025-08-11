use std::rc::Rc;
use raylib::prelude::*;
use crate::{framebuffer::Framebuffer, player::Player, texture::Texture};

#[derive(Clone)]
pub struct SpriteAnim {
    pub frames: Vec<Rc<Texture>>,
    pub fps: f32,
}
impl SpriteAnim {
    #[inline]
    pub fn frame_at(&self, t: f32) -> &Texture {
        let n = self.frames.len().max(1);
        let idx = ((t * self.fps) as usize) % n;
        &self.frames[idx]
    }
}

pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub size: f32,         // alto proyectado a 1.0 escala
    pub anim: SpriteAnim,
    // --- parámetros de “temblor” (px y Hz) ---
    pub wobble_amp: f32,   // amplitud en PIXELES de pantalla
    pub wobble_freq: f32,  // frecuencia en Hz
    pub phase: f32,        // desfase para no sincronizar
}

pub fn render_sprites(
    fb: &mut Framebuffer,
    player: &Player,
    sprites: &[Sprite],
    zbuf: &[f32],
    fov: f32,
    tsec: f32,
) {
    let hw = fb.width  as f32 / 2.0;
    let hh = fb.height as f32 / 2.0;
    let proj = hw / (fov / 2.0).tan();

    let mut order: Vec<usize> = (0..sprites.len()).collect();
    order.sort_by(|&a, &b| {
        let da = (sprites[a].x - player.pos.x).hypot(sprites[a].y - player.pos.y);
        let db = (sprites[b].x - player.pos.x).hypot(sprites[b].y - player.pos.y);
        db.partial_cmp(&da).unwrap()
    });

    for &i in &order {
        let s = &sprites[i];

        let dx = s.x - player.pos.x;
        let dy = s.y - player.pos.y;

        // Ángulo hacia el sprite y delta respecto a la vista
        let ang_to = dy.atan2(dx);
        let mut delta = ang_to - player.a;
        while delta >  std::f32::consts::PI { delta -= 2.0*std::f32::consts::PI; }
        while delta < -std::f32::consts::PI { delta += 2.0*std::f32::consts::PI; }
        if delta.abs() > fov { continue; }

        // Distancia perpendicular (profundidad real)
        let dist = dx * player.a.cos() + dy * player.a.sin();
        if dist <= 1.0 { continue; }

        // Tamaño proyectado
        let size = (proj / dist) * s.size;
        let w = size as i32;
        let h = size as i32;

        // Centro X en pantalla
        let mut x_center = hw + (delta / (fov / 2.0)) * hw;

        // --- wobble en pantalla (pequeño jitter “natural”) ---
        let omega = 2.0 * std::f32::consts::PI * s.wobble_freq;
        let wobx = s.wobble_amp * (omega * tsec + s.phase).sin();
        let woby = (s.wobble_amp * 0.6) * (omega * tsec * 1.1 + s.phase * 0.7).sin();
        x_center += wobx;

        let ground_bias = (h as f32) * 0.70; // ~40% de su alto hacia abajo
        let x0 = (x_center - w as f32 / 2.0).round() as i32;
        let x1 = (x_center + w as f32 / 2.0).round() as i32;
        let y0 = (hh - h as f32 / 2.0 + woby + ground_bias).round() as i32;
        let y1 = (hh + h as f32 / 2.0 + woby + ground_bias).round() as i32;



        let tex = s.anim.frame_at(tsec);
        let tw = tex.w as i32;
        let th = tex.h as i32;

        for xs in x0.max(0)..=x1.min(fb.width as i32 - 1) {
            // depth-test con paredes
            let zb = zbuf.get(xs as usize).copied().unwrap_or(f32::INFINITY);
            if dist >= zb { continue; }

            let u = (xs - x0) as f32 / (x1 - x0).max(1) as f32;
            for ys in y0.max(0)..=y1.min(fb.height as i32 - 1) {
                let v = (ys - y0) as f32 / (y1 - y0).max(1) as f32;

                let tx = ((u * tex.w as f32) as i32).clamp(0, tw - 1);
                let ty = ((v * tex.h as f32) as i32).clamp(0, th - 1);
                let pix = tex.px[(ty as u32 * tex.w + tx as u32) as usize];

                let a = ((pix >> 24) & 0xFF) as u8;
                if a < 10 { continue; }

                let r = ((pix >> 16) & 0xFF) as u8;
                let g = ((pix >> 8)  & 0xFF) as u8;
                let b = ( pix        & 0xFF) as u8;

                fb.set_current_color(Color::new(r, g, b, a));
                fb.set_pixel(xs as u32, ys as u32);
            }
        }
    }
}
