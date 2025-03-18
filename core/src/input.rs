use sdl2::keyboard::Keycode;

pub struct Input {
    keys: [bool; 16],
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: [false; 16]
        }
    }

    pub fn key_pressed(&mut self, keycode: Keycode) {
        // TODO Key press method
    }
}