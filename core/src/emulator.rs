use sdl2::Sdl;

use crate::{errors::{Error, Result}, screen::{SCREEN_HEIGHT, SCREEN_WIDTH}};

const WINDOW_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * WINDOW_SCALE;

pub struct Emulator {
    sdl: Sdl,
}

impl Emulator {
    pub fn new() -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
        
        todo!()
    }
}
