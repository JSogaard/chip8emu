use sdl2::keyboard::Keycode;

use crate::helpers::keycode_to_button;

const KEYCODES: [Keycode; 16] = [
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Num4,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::R,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::F,
    Keycode::Z,
    Keycode::X,
    Keycode::C,
    Keycode::V,
];

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

    pub fn check_key(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn check_all_keys(&self) -> Option<u8> {
        for (i, key_pressed) in self.keys.iter().enumerate() {
            if *key_pressed {
                return Some(i as u8);
            }
        }
        // If not key is pressed
        None
    }
}
