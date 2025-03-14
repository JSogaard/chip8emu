use crate::errors::Error;
use crate::errors::Result;

pub const RAM_SIZE: usize = 4096;
pub const NUM_REGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const START_ADDR: u16 = 0x200;
pub const MAX_ROM_SIZE: usize = RAM_SIZE - START_ADDR as usize;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const FONTSET_SIZE: usize = 16 * 5;
pub const FONTSET_ADDR: u16 = 0x050;
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
    // Program counter
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    // General purpose registers
    v_reg: [u8; NUM_REGS],
    // Index register
    i_reg: u16,
    // Stack pointer
    sp: u16,
    stack: [u16; STACK_SIZE],
    // Sound timer
    st: u8,
    // Delay timer
    dt: u8,
    redraw_flag: bool,
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
            redraw_flag: false,
        };

        // Copying font set into ram from address 0x50 (80)
        // Get target location in RAM as slice and copy font set to it
        emulator.ram[FONTSET_ADDR as usize..(FONTSET_ADDR as usize + FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        emulator
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() <= MAX_ROM_SIZE {
            self.ram[START_ADDR as usize..].copy_from_slice(rom);
        } else {
            return Err(Error::InvalidRomSizeError);
        }

        Ok(())
    }

    pub fn redraw_flag(&self) -> bool {
        self.redraw_flag
    }

    pub fn reset(&mut self) {
        self.pc = 0;
    }

    pub fn cycle(&mut self) -> Result<bool> {
        // Returns bool draw flag

        self.redraw_flag = false;
        // Get opcode as u16
        let low_byte = self.ram[self.pc as usize] as u16;
        let high_byte = self.ram[(self.pc + 1) as usize] as u16;
        let opcode = (low_byte << 8) | high_byte;
        self.pc += 2;

        // DECODE AND EXECUTE OPCODE
        // Filter op code to match only the first half byte
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(),
                0x00EE => self.return_subroutine(),
                // If op code is 0NNN - call machine code subroutine,
                // which isn't implemented.
                _ => {
                    return Err(Error::InvalidOpcodeError(
                        "0NNN - Call machine code routine".into(),
                    ))
                }
            },

            0x1000 => self.jump(opcode),
            0x2000 => self.call_subroutine(opcode)?,
            0x3000 => self.skip_equal(opcode),
            0x4000 => self.skip_not_equal(opcode),
            0x5000 => self.skip_register_equal(opcode),
            0x6000 => self.load_number(opcode),
            0x7000 => self.add_number(opcode),

            // Register loading op codes
            0x8000 => match opcode & 0x000F {
                0x0 => self.load_register_op(opcode, |_, vy| vy),
                0x1 => self.load_register_op(opcode, |vx, vy| vx | vy),
                0x2 => self.load_register_op(opcode, |vx, vy| vx & vy),
                0x3 => self.load_register_op(opcode, |vx, vy| vx ^ vy),
                _ => return Err(Error::UnknownOpcodeError(opcode))
            },

            _ => return Err(Error::UnknownOpcodeError(opcode))
        }

        todo!()
    }

    fn push(&mut self, val: u16) -> Result<()> {
        // Check for stack overflow
        if self.sp >= STACK_SIZE as u16 {
            return Err(Error::StackOverflowError);
        }

        self.stack[self.sp as usize] = val;
        self.sp += 1;

        Ok(())
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    //************************************************************//
    //                      OPCODE METHODS                        //
    //************************************************************//

    /// Opcode 00E0
    /// Clear screen
    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.redraw_flag = true;
    }

    /// Opcode 00EE
    /// Return from subroutine
    fn return_subroutine(&mut self) {
        // Pop return address from stack and set PC to it
        let return_address = self.pop();
        self.pc = return_address;
    }

    /// Opcode 1NNN
    /// Jump to address NNN
    fn jump(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.pc = address;
    }

    /// Opcode 2NNN
    /// Call subroutine at address NNN
    fn call_subroutine(&mut self, opcode: u16) -> Result<()> {
        // PC is pushed to stack to remember where to return after subroutine
        self.push(self.pc)?;
        let address = opcode & 0x0FFF;
        self.pc = address;

        Ok(())
    }

    /// Opcode 3XNN
    /// Skip next instruction if VX == NN
    fn skip_equal(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        if number == self.v_reg[register] {
            self.pc += 2;
        }
    }

    /// Opcode 4XNN
    /// Skip next instruction if VX != NN
    fn skip_not_equal(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        if number != self.v_reg[register] {
            self.pc += 2;
        }
    }

    /// Opcode 5XY0
    /// Skip next instruction if VX == VY
    fn skip_register_equal(&mut self, opcode: u16) {
        let register_x = ((opcode & 0x0F00) >> 8) as usize;
        let register_y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v_reg[register_x] == self.v_reg[register_y] {
            self.pc += 2;
        }
    }

    /// Opcode 6XNN
    /// Load NN into VX
    fn load_number(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        self.v_reg[register] = number;
    }

    /// Opcode 7XNN
    /// Add NN to VX (VX += NN)
    fn add_number(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        self.v_reg[register] += number;
    }

    /// Opcode 8XY1 to 8XY3
    /// Load op(VX, VY) into VX
    fn load_register_op<F: Fn(u8, u8) -> u8>(&mut self, opcode: u16, op: F) {
        let register_x = ((opcode & 0x0F00) >> 8) as usize;
        let register_y = ((opcode & 0x00F0) >> 4) as usize;
        let value = op(self.v_reg[register_x], self.v_reg[register_y]);
        self.v_reg[register_x] = value;
    }

    /// Opcode 8XY4
    /// Add value of VY to VX (VX += VY) and enable VF if overflow
    fn add_register_carry(&mut self, opcode: u16) {
        let register_x = ((opcode & 0x0F00) >> 8) as usize;
        let register_y = ((opcode & 0x00F0) >> 4) as usize;
        let result = self.v_reg[register_x] as u16 + self.v_reg[register_y] as u16;
        self.v_reg[register_x] = result as u8;

        // Enable carry register if addition overflows
        self.v_reg[NUM_REGS - 1] = (result > 255) as u8;
    }
}
