use sdl2::{event::Event, keyboard::Keycode, EventPump, Sdl};
use std::{
    fs::File,
    io::Read,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    display::Display,
    errors::{Error, Result},
    input::Input,
    processor::Processor,
};

const FRAME_RATE: u32 = 60;
const CLOCK_SPEED: u32 = 600;
const CYCLES_PER_FRAME: u32 = CLOCK_SPEED / FRAME_RATE + 1;

pub struct Emulator {
    processor: Processor,
    display: Display,
    input: Input,
    sdl_context: Sdl,
    event_pump: EventPump,
}

impl Emulator {
    pub fn new(rom_path: &str, window_scale: u32) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(Error::SdlError)?;
        let video_subsystem = sdl_context.video().map_err(Error::SdlError)?;

        let mut rom_file = File::open(rom_path)?;
        let mut rom = Vec::new();
        rom_file.read_to_end(&mut rom)?;

        let event_pump = sdl_context.event_pump().map_err(Error::SdlError)?;

        // IMPL set up beep

        Ok(Self {
            processor: Processor::new(&rom)?,
            display: Display::new(video_subsystem, window_scale)?,
            input: Input::new(),
            sdl_context,
            event_pump,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let frame_length = Duration::from_secs_f64(1. / FRAME_RATE as f64);

        'main_loop: loop {
            let frame_start = Instant::now();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'main_loop;
                    }

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        self.input.key_press(keycode);
                    }

                    _ => {}
                }
            }

            // Run CPU cycles
            for _ in 0..CYCLES_PER_FRAME {
                self.processor.cycle(&mut self.display, &mut self.input)?;
            }

            if self.display.redraw_needed() {
                self.display.render()?;
            }

            // IMPL Check sount timer and make beep

            // Frame timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_length {
                sleep(frame_length - elapsed);
            }
        }

        Ok(())
    }
}
