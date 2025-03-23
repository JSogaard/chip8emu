use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, VideoSubsystem};

use crate::{errors::Error, errors::Result, helpers::bit_to_bool};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const BACKGROUND_COLOR: Color = Color::RGB(0, 75, 0);
const FOREGROUND_COLOR: Color = Color::RGB(0, 255, 0);

pub struct Display {
    pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    canvas: Canvas<Window>,
    window_scale: u32,
    window_width: u32,
    window_height: u32,
    redraw_flag: bool,
}

impl Display {
    pub fn new(video_subsystem: VideoSubsystem, window_scale: u32) -> Result<Self> {
        let window_width = (SCREEN_WIDTH as u32) * window_scale;
        let window_height = (SCREEN_HEIGHT as u32) * window_scale;

        let mut canvas = video_subsystem
            .window("CHIP-8 Emulator", window_width, window_height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| Error::SdlError(e.to_string()))?
            .into_canvas()
            .build()
            .map_err(|e| Error::SdlError(e.to_string()))?;

        canvas.clear();
        canvas.present();

        Ok(Self {
            pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            canvas,
            window_scale,
            window_width,
            window_height,
            redraw_flag: false,
        })
    }

    pub fn redraw_needed(&self) -> bool {
        self.redraw_flag
    }

    pub fn draw(&mut self, sprite: &[u8], x_coord: u8, y_coord: u8) -> u8 {
        self.redraw_flag = true;

        let mut carry_register: u8 = 0x0;

        for (k, sprite_byte) in sprite.iter().enumerate() {
            let y_pos = y_coord + k as u8;
            if y_pos as usize >= SCREEN_HEIGHT {
                // If reaching bottom edge of display, break loop
                break;
            }

            for j in 0..8 {
                let sprite_pixel = bit_to_bool(*sprite_byte, j);
                // Index of pixel on screen
                let x_pos = x_coord + j;
                let pixel_index = y_pos as usize * SCREEN_WIDTH + x_pos as usize;

                if x_pos as usize >= SCREEN_WIDTH {
                    // If reaching right edge of screen, continue to next row
                    break;
                } else if self.pixels[pixel_index] && sprite_pixel {
                    // If the pixel on screen and in sprite
                    // are on then turn off screen pixel
                    self.pixels[pixel_index] = false;
                    carry_register = 0x1;
                } else if sprite_pixel {
                    // Else if sprite pixel is on but screen pixel is not
                    // turn on screen pixel
                    self.pixels[pixel_index] = true;
                }
            }
        }
        carry_register
    }

    pub fn clear(&mut self) {
        self.pixels = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    pub fn render(&mut self) -> Result<()> {
        let scale_usize = self.window_scale as usize;

        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();

        self.canvas.set_draw_color(FOREGROUND_COLOR);
        for (i, pixel) in self.pixels.iter().enumerate() {
            if *pixel {
                let x = (i % SCREEN_WIDTH * scale_usize) as i32;
                let y = (i / SCREEN_WIDTH * scale_usize) as i32;
                let rect = Rect::new(x, y, self.window_scale, self.window_scale);
                self.canvas.fill_rect(rect).map_err(Error::SdlError)?;
            }
        }
        self.canvas.present();

        self.redraw_flag = false;
        Ok(())
    }
}
