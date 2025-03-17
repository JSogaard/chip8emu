use sdl2::{render::Canvas, video::Window, VideoSubsystem};

use crate::helpers::bit_to_bool;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const WINDOW_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * WINDOW_SCALE;

pub struct Display {
    pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    canvas: Canvas<Window>,
    redraw_flag: bool,
}

impl Display {
    pub fn new(video_subsystem: VideoSubsystem) -> Self {
        let mut canvas = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap();

    canvas.clear();
    canvas.present();

    // TODO Finish Display constructor

        Self {
            pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            canvas,
            redraw_flag: false,
        }
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

    pub fn render(&mut self) {
        // TODO Create render method
    }
}