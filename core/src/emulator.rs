use sdl2::{event::Event, keyboard::Keycode, EventPump, Sdl};
use std::{
    fs,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    audio_output::AudioOutput, display::Display, errors::{Error, Result}, key_input::KeyInput, processor::Processor
};

const FRAME_RATE: u32 = 60;
const CLOCK_SPEED: u32 = 600;
const CYCLES_PER_FRAME: u32 = CLOCK_SPEED / FRAME_RATE + 1;

pub struct Emulator {
    processor: Processor,
    display: Display,
    input: KeyInput,
    audio: AudioOutput,
    _sdl_context: Sdl,
    event_pump: EventPump,
}

impl Emulator {
    pub fn try_new(rom_path: &str, window_scale: u32) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(Error::SdlError)?;
        let video_subsystem = sdl_context.video().map_err(Error::SdlError)?;

        let rom: Vec<u8> = fs::read(rom_path)?;

        let event_pump = sdl_context.event_pump().map_err(Error::SdlError)?;

        Ok(Self {
            processor: Processor::try_new(&rom)?,
            display: Display::try_new(video_subsystem, window_scale)?,
            input: KeyInput::new(),
            audio: AudioOutput::try_new()?,
            _sdl_context: sdl_context,
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
                    },

                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        self.input.key_press(keycode);
                    },

                    Event::KeyUp {keycode: Some(keycode), .. } => self.input.key_release(keycode),

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

            self.processor.tick_timers();

            if self.processor.check_beep() {
                self.audio.start()
            } else {
                self.audio.stop();
            }

            // Frame timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_length {
                sleep(frame_length - elapsed);
            }
        }

        Ok(())
    }
}
