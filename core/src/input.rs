use sdl2::keyboard::Keycode;

use crate::helpers::keycode_to_button;

pub struct Input {
    keys: [bool; 16],
}

impl Input {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn key_press(&mut self, keycode: Keycode) {
        if let Some(key_number) = keycode_to_button(keycode) {
            self.keys[key_number] = true;
        }
    }

    pub fn check_key(&mut self, key_number: u8) -> bool {
        let key = self.keys[key_number as usize];
        self.keys[key_number as usize] = false;
        key
    }

    pub fn check_all_keys(&mut self) -> Option<u8> {
        for (i, key_pressed) in self.keys.iter().enumerate() {
            if *key_pressed {
                self.reset();
                return Some(i as u8);
            }
        }
        // If not key is pressed
        None
    }

    fn reset(&mut self) {
        self.keys = [false; 16];
    }
}
