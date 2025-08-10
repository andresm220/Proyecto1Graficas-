// src/textures.rs
use std::collections::HashMap;
use std::rc::Rc;
use crate::texture::Texture;

pub struct TextureAtlas {
    default: Rc<Texture>,
    map: HashMap<char, Rc<Texture>>,
}

impl TextureAtlas {
    pub fn new(default: Rc<Texture>) -> Self {
        Self { default, map: HashMap::new() }
    }

    pub fn insert(&mut self, tile: char, tex: Rc<Texture>) {
        self.map.insert(tile, tex);
    }

    pub fn get(&self, tile: char) -> &Texture {
        self.map.get(&tile).unwrap_or(&self.default).as_ref()
    }
}
