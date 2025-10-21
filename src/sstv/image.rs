use image::{ImageBuffer, Rgb};

pub type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub struct ImageBuilder {
    pub img: Image,
    pub y: u32,
    pub sample_rate: u32,
}

impl ImageBuilder {
    pub fn new(sample_rate: u32, width: u32, height: u32) -> Self {
        ImageBuilder {
            img: ImageBuffer::new(width, height),
            sample_rate,
            y: 0,
        }
    }

    pub fn push_row(&mut self, row: &[f32]) {
        let samples_per_row = 0.48 * self.sample_rate as f32;
        let rows = (row.len() as f32 / samples_per_row).round() as usize;
        if rows == 0 {
            return;
        }

        for row in row.chunks(row.len() / rows) {
            if self.y >= self.img.height() {
                return;
            }

            let get = |x: f32| {
                let idx = row.len() as f32 * x;

                let prev = row[idx as usize];
                let next = row[(idx.ceil() as usize).min(row.len() - 1)];
                (lerp(prev, next, idx.fract()) * 255.0) as u8
            };

            let width = self.img.width();
            for x in 0..width {
                let t = x as f32 / width as f32 / 3.0;
                let color = Rgb([get(t + 2. / 3.), get(t), get(t + 1. / 3.)]);
                // let color = Rgb([get(t), get(t), get(t)]);
                self.img.put_pixel(x, self.y, color);
            }

            self.y += 1;
        }
    }

    pub fn finished(&self) -> bool {
        self.y >= self.img.height()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
