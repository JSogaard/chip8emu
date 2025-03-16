use sdl2::Sdl;

use crate::{errors::{Error, Result}, helpers::bit_to_bool};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Screen {
    sdl: Sdl,
    pixels: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Screen {
    pub fn new() -> Result<Self> {
        let sdl = match sdl2::init() {
            Ok(sdl) => sdl,
            Err(_) => return Err(Error::ScreenInitError),
        };

        let screen = Self {
            sdl,
            pixels: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        };
        Ok(screen)
    }

    pub fn draw(&mut self, sprite: &[u8], x_coord: u8, y_coord: u8) -> u8 {
        let mut carry_register: u8 = 0x0;

        for (k, sprite_byte) in sprite.iter().enumerate() {

            let y_pos = y_coord + k as u8;
            if y_pos as usize >= SCREEN_HEIGHT {
                // If reaching bottom edge of screen, break loop
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
}