use sdl2::keyboard::Keycode;

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
        // TODO Key press method
    }

    pub fn check_key(&self, key: u8) -> bool {
        // TODO Check key method
        todo!()
    }

    pub fn check_all_keys(&self) -> Option<u8> {
        // Check all keys method
        todo!()
    }
}
