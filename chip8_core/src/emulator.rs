pub const RAM_SIZE: usize = 4096;
pub const NUM_REGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const START_ADDR: usize = 0x200;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const FONTSET_SIZE: usize = 16 * 5;
pub const FONTSET_ADDR: usize = 0x050;
pub const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    pc: usize,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: usize,
    stack: [u16; STACK_SIZE],
    st: u8,
    dt: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            st: 0,
            dt: 0,
        };

        // Copying font set into ram from address 0x50 (80)
        // Get target location in RAM as slice and copy font set to it
        emulator.ram[FONTSET_ADDR..(FONTSET_ADDR + FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        emulator
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    fn load_rom(&mut self, rom: &[u8]) {
        self.ram[START_ADDR..].copy_from_slice(rom);
    }

    fn cycle(&mut self) -> bool {
        // Returns bool draw flag

        // Get opcode as u16
        let low_byte = self.ram[self.pc] as u16;
        let high_byte = self.ram[self.pc + 1] as u16;
        let op_code = (low_byte << 8) | high_byte;
        self.pc += 2;

        match op_code & 0xF000 {
            0x1000 => {
                self.jump(op_code);
            }
        }

        todo!()
    }

    // INSTRUCTION FUNCTIONS

    fn jump(&mut self, op_code: u16) {
        todo!()
    }
}
