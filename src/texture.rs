use image::{GenericImageView, ImageError};

pub struct Texture {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u32>, // 0xAARRGGBB
}

impl Texture {
    pub fn from_file(path: &str) -> Result<Self, ImageError> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let mut px = Vec::with_capacity((w * h) as usize);
        for p in rgba.pixels() {
            let [r, g, b, a] = p.0;
            px.push(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
        }
        Ok(Self { w, h, px })
    }

    #[inline]
    pub fn sample(&self, mut u: f32, mut v: f32) -> u32 {
        // wrap 0..1
        u = u - u.floor();
        v = v - v.floor();
        let x = (u * self.w as f32) as u32;
        let y = (v * self.h as f32) as u32;
        let idx = (y.min(self.h - 1) * self.w + x.min(self.w - 1)) as usize;
        self.px[idx]
    }
}
