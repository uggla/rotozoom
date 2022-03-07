#![allow(dead_code)]
use std::f64::consts::PI;

use macroquad::prelude::*;

struct Effect<'a> {
    curtain_width: u32,
    src_image: &'a mut Image,
    dst_image: &'a mut Image,
}

impl<'a> Effect<'a> {
    fn new(src_image: &'a mut Image, dst_image: &'a mut Image) -> Self {
        Self {
            curtain_width: 0,
            src_image,
            dst_image,
        }
    }

    fn left_curtain(&mut self, color: Color) {
        if self.curtain_width < self.dst_image.width() as u32 {
            for _ in 0..5 {
                for h in 0..self.dst_image.height() {
                    self.dst_image
                        .set_pixel(self.curtain_width as u32, h as u32, color);
                }
                self.curtain_width += 1;
            }
        }
    }

    fn set_color(&mut self, color: Color) {
        for w in 0..self.dst_image.width() {
            for h in 0..self.dst_image.height() {
                self.dst_image.set_pixel(w as u32, h as u32, color);
            }
            self.curtain_width += 1;
        }
    }

    fn make_transparent(&mut self) {
        const STEP: u8 = 5;
        let mut count = 0;
        // Iterate on image bytes and decrease alpha channel (every 4 bytes) to 0
        for byte in self.dst_image.bytes.iter_mut() {
            if count == 3 {
                if *byte > STEP {
                    *byte -= STEP;
                } else {
                    *byte = 0;
                }
                count = 0;
            } else {
                count += 1;
            }
        }
    }

    fn rotozoom(&mut self, angle: f64, zoom: f64) {
        let dx: f64 = angle.cos() * zoom;
        let dy: f64 = angle.sin() * zoom;

        let mut src_x: f64 = -dx * ((self.dst_image.width() / 2) as f64);
        let mut src_y: f64 = -dy * ((self.dst_image.width() / 2) as f64);

        let mut src_x_start: f64;
        let mut src_y_start: f64;

        src_x += dy * (self.dst_image.height() / 2) as f64;
        src_y -= dx * (self.dst_image.height() / 2) as f64;

        for dst_y in 0..self.dst_image.height() {
            src_x_start = src_x;
            src_y_start = src_y;
            for dst_x in 0..self.dst_image.width() {
                let (x, y) = wrap(
                    src_x as isize,
                    src_y as isize,
                    self.src_image.width() as isize,
                    self.src_image.height() as isize,
                );

                self.copy_pixel(x, y, dst_x, dst_y);
                // let color = self.src_image.get_pixel(x as u32, y as u32);
                // self.dst_image.set_pixel(dst_x as u32, dst_y as u32, color);

                src_x += dx;
                src_y += dy;
            }
            src_x = src_x_start - dy;
            src_y = src_y_start + dx;
        }
    }

    fn copy_pixel(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize) {
        let src_offset = src_y * self.src_image.width() as usize * 4 + src_x * 4;
        let dst_offset = dst_y * self.dst_image.width() as usize * 4 + dst_x * 4;
        self.dst_image.bytes[dst_offset..(dst_offset + 4)]
            .copy_from_slice(&self.src_image.bytes[src_offset..(src_offset + 4)]);
    }
}

fn wrap(mut x: isize, mut y: isize, sx: isize, sy: isize) -> (usize, usize) {
    while x < 0 {
        x += sx;
    }

    while x >= sx {
        x -= sx;
    }

    while y < 0 {
        y += sy;
    }

    while y >= sy {
        y -= sy;
    }
    (x as usize, y as usize)
}

fn display_fps(fps: &mut i32, frame_t: f64, fps_refresh: &mut f64) {
    if frame_t - *fps_refresh > 0.2 {
        *fps = get_fps();
        *fps_refresh = frame_t;
    }
    let text = format!("{} fps", fps);
    let font_size = 30.;
    draw_text(&text, 5., 20., font_size, DARKGRAY)
}

fn display_end_msg() {
    let text = "That's all folks !".to_string();
    let font_size = 80.;
    let text_size = measure_text(&text, None, font_size as _, 1.0);
    draw_text(
        &text,
        screen_width() / 2. - text_size.width / 2.,
        screen_height() / 2. - text_size.height / 2.,
        font_size,
        SKYBLUE,
    )
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Rotozoom"),
        window_width: 1280,
        window_height: 800,
        window_resizable: false,

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut fps: i32 = 0;
    let mut fps_t = get_time();
    let mut src_image: Image = load_image("src/ferris.png").await.unwrap();
    let mut dst_image =
        Image::gen_image_color(screen_width() as u16, screen_height() as u16, BLACK);
    let texture: Texture2D = Texture2D::from_image(&dst_image);

    let mut effect = Effect::new(&mut src_image, &mut dst_image);
    let close_event_t = get_time();

    let mut angle: f64 = 0.0;
    const ROT_ANGRE: f64 = 4.0;

    loop {
        let frame_t = get_time();
        let zoom = (angle / 2.0).cos() + 1.5;
        angle += ROT_ANGRE * PI / 180.0;
        if angle >= 4.0 * PI {
            angle = 0.0;
        }
        clear_background(WHITE);
        draw_texture(
            texture,
            screen_width() / 2. - texture.width() / 2.,
            screen_height() / 2. - texture.height() / 2.,
            WHITE,
        );

        if (frame_t - close_event_t) > 30.0 && angle == 0.0 {
            angle = 20.0; // set a high value to be reset to 0.0 in next loop
            effect.left_curtain(WHITE);
            //effect.make_transparent();
            if (frame_t - close_event_t) > 35.0 {
                display_end_msg();
            }
            if (frame_t - close_event_t) > 38.0 {
                break;
            }
        } else {
            effect.rotozoom(angle, zoom);
        }
        texture.update(effect.dst_image);
        // dbg!(get_fps());
        display_fps(&mut fps, frame_t, &mut fps_t);
        next_frame().await
    }
}
