use sdl2::{EventPump, Sdl};
use std::{fs::File, io::Read};

use crate::{
    processor::Processor,
    display::Display,
    errors::Result,
};

const FRAME_RATE: u32 = 60;
const CLOCK_SPEED: u32 = 500;
const CYCLES_PER_FRAME: u32 = CLOCK_SPEED / FRAME_RATE + 1;

pub struct Emulator {
    processor: Processor,
    display: Display,
    sdl_context: Sdl,
    event_pump: EventPump,
}

impl Emulator {
    pub fn new(rom_path: &str) -> Result<Self> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let mut rom_file = File::open(rom_path)?;
        let mut rom = Vec::new();
        rom_file.read_to_end(&mut rom)?;

        let event_pump = sdl_context.event_pump().unwrap();

        Ok(Self {
            processor: Processor::new(&rom)?,
            display: Display::new(video_subsystem),
            sdl_context,
            event_pump,
        })
    }

    pub fn run() {
        // TODO Game loop
    }
}
