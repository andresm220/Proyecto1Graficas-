use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2,
    pub a:   f32,   // ángulo en radianes
    pub fov: f32,   // campo de visión (radianes)
}

impl Player {
     pub fn new(x: f32, y: f32, angle: f32, fov: f32) -> Self {
       Self {
           pos: Vector2::new(x, y),
           a:   angle,   // asignamos angle al campo a
           fov,
       }
     }
 }