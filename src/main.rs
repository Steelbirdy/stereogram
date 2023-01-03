mod text;

use image::{Rgb, RgbImage};
use rand::Rng;

fn main() {
    let depth_map = image::open("depth.png").unwrap();
    let depth_map = depth_map.as_rgb8().unwrap();

    let texture = image::open("texture.png").unwrap();
    let texture = texture.as_rgb8().unwrap();
    let texture = image::imageops::resize(texture, depth_map.width() / 8, depth_map.height(), image::imageops::FilterType::Gaussian);

    let ret = draw_auto_stereogram(depth_map, ImageTexture { img: &texture });
    ret.save("output.png").unwrap();
}

fn separation(x: f64, e: f64, mu: f64) -> f64 {
    f64::round((1. - mu * x) * e / (2. - mu * x))
}

pub trait Texture {
    fn get(&mut self, x: u32, y: u32) -> Rgb<u8>;

    fn size(&self) -> Option<(u32, u32)>;
}

impl<T: Texture> Texture for &'_ mut T {
    fn get(&mut self, x: u32, y: u32) -> Rgb<u8> {
        T::get(self, x, y)
    }

    fn size(&self) -> Option<(u32, u32)> {
        T::size(self)
    }
}

pub struct Random<R>(R);

impl<R: Rng> Texture for Random<R> {
    fn get(&mut self, _: u32, _: u32) -> Rgb<u8> {
        let p: u8 = self.0.gen_range(0..=1) * 255;
        Rgb([p; 3])
    }

    fn size(&self) -> Option<(u32, u32)> {
        None
    }
}

pub struct ImageTexture<'a> {
    img: &'a RgbImage,
}

impl Texture for ImageTexture<'_> {
    fn get(&mut self, x: u32, y: u32) -> Rgb<u8> {
        *self.img.get_pixel(x, y)
    }

    fn size(&self) -> Option<(u32, u32)> {
        Some(self.img.dimensions())
    }
}

pub fn draw_auto_stereogram(z: &RgbImage, mut texture: impl Texture) -> RgbImage {
    let pixel = |x: u32, y: u32| z.get_pixel(x, y).0[0] as f64 / 255.0;

    let dpi: f64 = 72.0;
    let e = (2.5 * dpi).round();
    let mu = 1. / 5.;

    let (max_x, max_y) = z.dimensions();
    let (tw, th) = texture.size().unwrap_or((max_x, max_y));

    let mut ret = RgbImage::new(max_x as _, max_y as _);

    for y in 0..max_y {
        let mut px = vec![Rgb([0_u8; 3]); max_x as _].into_boxed_slice();
        let mut same: Box<[_]> = (0..max_x).collect();

        for x in 0..max_x {
            let s = separation(pixel(x, y), e, mu);
            let left = (x as f64) - s / 2.;
            let right = left + s;
            if 0. <= left && right < max_x as f64 {
                let mut left = left as u32;
                let mut right = right as u32;

                let mut l = same[left as usize];
                while l != left && l != right {
                    if l < right {
                        left = l;
                        l = same[left as usize];
                    } else {
                        same[left as usize] = right;
                        left = right;
                        l = same[left as usize];
                        right = l;
                    }
                }
                same[left as usize] = right;
            }
        }
        for x in (0..max_x).rev() {
            let p = if same[x as usize] == x {
                let mut i = x % (tw * 2);
                if i > tw {
                    i -= tw;
                }
                let j = y % th;
                texture.get(i, j)
            } else {
                px[same[x as usize] as usize]
            };
            px[x as usize] = p;
            *ret.get_pixel_mut(x, y) = p;
        }
    }

    ret
}
