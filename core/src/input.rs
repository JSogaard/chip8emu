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

    pub fn key_press(&mut self, keycode: Keycode) {
        // TODO Key press method
    }

    pub fn check_key(&self, key: u8) -> bool {
        // TODO Check key method
        todo!()
    }
}