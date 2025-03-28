use sdl2::keyboard::Keycode;

use crate::helpers::keycode_to_button;

pub struct KeyInput {
    keys: [bool; 16],
}

impl KeyInput {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn key_press(&mut self, keycode: Keycode) {
        if let Some(key_number) = keycode_to_button(keycode) {
            self.keys[key_number] = true;
        }
    }

    pub fn key_release(&mut self, keycode: Keycode) {
        if let Some(key_number) = keycode_to_button(keycode) {
            self.keys[key_number] = false;
        }
    }

    pub fn check_key(&mut self, key_number: u8) -> bool {
        self.keys[key_number as usize]
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

    pub fn reset(&mut self) {
        self.keys = [false; 16];
    }
}
