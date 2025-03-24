use rand::Rng;

use crate::display::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::errors::{Error, Result};
use crate::helpers::decode_middle_registers;
use crate::input::Input;
use crate::memory::{Memory, FONTSET_ADDR, RAM_SIZE, START_ADDR};
use crate::stack::Stack;

pub const NUM_REGS: usize = 16;
pub const CARRY_REGISTER: usize = NUM_REGS - 1;

#[derive(Debug)]
pub struct Processor {
    // Program counter
    pc: u16,
    memory: Memory,
    // General purpose registers
    v_reg: [u8; NUM_REGS],
    // Index register
    i_reg: u16,
    stack: Stack,
    // Sound timer
    st: u8,
    // Delay timer
    dt: u8,
    rng: rand::rngs::ThreadRng,
}

impl Processor {
    pub fn new(rom: &[u8]) -> Result<Self> {
        let mut memory = Memory::new();
        memory.load_rom(rom)?;

        Ok(Self {
            pc: START_ADDR,
            memory,
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: Stack::new(),
            st: 0,
            dt: 0,
            rng: rand::rng(),
        })
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.memory.reset();
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack.reset();
        self.st = 0;
        self.dt = 0;
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn check_beep(&self) -> bool {
        self.st > 0
    }

    pub fn cycle(&mut self, display: &mut Display, input: &mut Input) -> Result<()> {
        // Check if ROM as been loaded into RAM
        if !self.memory.rom_loaded() {
            return Err(Error::MissingRomError);
        }

        // Check if the end of RAM is reached
        if self.pc as usize >= RAM_SIZE {
            return Err(Error::InvalidRamAddressError);
        }

        // Get opcode as u16
        let high_byte = self.memory.read(self.pc) as u16;
        let low_byte = self.memory.read(self.pc + 1) as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.pc += 2;

        // DECODE AND EXECUTE OPCODE
        // Filter op code to match only the first half byte
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => display.clear(),
                0x00EE => self.return_subroutine()?,
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

            // Register loading opcodes
            0x8000 => match opcode & 0x000F {
                // Simple load instruction
                0x0 => self.load_register_op(opcode, |_, vy| vy),
                // OR
                0x1 => self.load_register_op(opcode, |vx, vy| vx | vy),
                // AND
                0x2 => self.load_register_op(opcode, |vx, vy| vx & vy),
                // XOR
                0x3 => self.load_register_op(opcode, |vx, vy| vx ^ vy),
                0x4 => self.add_register_carry(opcode),
                0x5 => self.sub_register(opcode),
                0x6 => self.shift_right(opcode),
                0x7 => self.sub_register_reversed(opcode),
                0xE => self.shift_left(opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0x9000 => self.skip_register_not_equal(opcode),
            0xA000 => self.load_i(opcode),
            0xB000 => self.jump_plus(opcode),
            0xC000 => self.random_and(opcode),
            0xD000 => self.draw_sprite(opcode, display)?,

            0xE000 => match opcode & 0x00FF {
                0x9E => self.skip_if_keypress(opcode, input),
                0xA1 => self.skip_if_not_keypress(opcode, input),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0xF000 => match opcode & 0x00FF {
                0x07 => self.load_delay_timer(opcode),
                0x0A => self.wait_for_keypress(opcode, input),
                0x15 => self.set_delay_timer(opcode),
                0x18 => self.set_sound_timer(opcode),
                0x1E => self.load_add_i(opcode),
                0x29 => self.find_character(opcode),
                0x33 => self.store_bcd(opcode),
                0x55 => self.dump_registers_to_ram(opcode)?,
                0x65 => self.load_registers_from_ram(opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            }

            _ => return Err(Error::UnknownOpcodeError(opcode)),
        }

        Ok(())
    }

    fn set_carry(&mut self, value: u8) {
        self.v_reg[CARRY_REGISTER] = value;
    }

    fn set_reg(&mut self, register: u16, value: u8) {
        self.v_reg[register as usize] = value;
    }

    fn get_reg(&self, register: u16) -> u8 {
        self.v_reg[register as usize]
    }

    //************************************************************//
    //                      OPCODE METHODS                        //
    //************************************************************//

    /// Opcode 00E0
    /// Clear screen
    fn clear_display(&mut self, display: &mut Display) {
        display.clear();
    }

    /// Opcode 00EE
    /// Return from subroutine
    fn return_subroutine(&mut self) -> Result<()> {
        // Pop return address from stack and set PC to it
        let return_address = self.stack.pop()?;
        self.pc = return_address;

        Ok(())
    }

    /// Opcode 1NNN
    /// Jump to address NNN
    fn jump(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// Opcode 2NNN
    /// Call subroutine at address NNN
    fn call_subroutine(&mut self, opcode: u16) -> Result<()> {
        // PC is pushed to stack to remember where to return after subroutine
        //FIXME Fix call subroutine - ends up wrong place
        self.stack.push(self.pc)?;
        let address = opcode & 0x0FFF;
        self.pc = address;

        Ok(())
    }

    /// Opcode 3XNN
    /// Skip next instruction if VX == NN
    fn skip_equal(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let number = (opcode & 0x00FF) as u8;
        if number == self.get_reg(register) {
            self.pc += 2;
        }
    }

    /// Opcode 4XNN
    /// Skip next instruction if VX != NN
    fn skip_not_equal(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let number = (opcode & 0x00FF) as u8;
        if number != self.get_reg(register) {
            self.pc += 2;
        }
    }

    /// Opcode 5XY0
    /// Skip next instruction if VX == VY
    fn skip_register_equal(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        if self.get_reg(reg_x) == self.get_reg(reg_y) {
            self.pc += 2;
        }
    }

    /// Opcode 6XNN
    /// Load NN into VX
    fn load_number(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let number = (opcode & 0x00FF) as u8;
        self.set_reg(register, number);
    }

    /// Opcode 7XNN
    /// Add NN to VX (VX += NN)
    fn add_number(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let number = (opcode & 0x00FF) as u8;
        let result = self.get_reg(register).wrapping_add(number);
        self.set_reg(register, result);
    }

    /// Opcode 8XY1 to 8XY3
    /// Load op(VX, VY) into VX
    fn load_register_op<F: Fn(u8, u8) -> u8>(&mut self, opcode: u16, op: F) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let result = op(self.get_reg(reg_x), self.get_reg(reg_y));
        self.set_reg(reg_x, result);
    }

    /// Opcode 8XY4
    /// Add value of VY to VX (VX += VY) and enable carry register if overflowing
    fn add_register_carry(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let result = self.get_reg(reg_x).wrapping_add(self.get_reg(reg_y));
        let carry = (result < self.get_reg(reg_x)) as u8;
        self.set_reg(reg_x, result);

        // Enable carry register if addition overflows
        self.set_carry(carry);
    }

    /// Opcode 8XY5
    /// Subtract value of VY from VX (VX -= VY) and enable carry register
    /// if not borrowing
    fn sub_register(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);

        // Enable carry register if subtraction borrows
        let not_borrow = (self.get_reg(reg_x) >= self.get_reg(reg_y)) as u8;

        let result = self.get_reg(reg_x).wrapping_sub(self.get_reg(reg_y));
        self.set_reg(reg_x, result);
        
        self.set_carry(not_borrow);
    }

    /// Opcode 8XY6
    /// Set carry register to least significant bit of VX
    /// and shift VX one bit right
    fn shift_right(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        // Quirk set VX to value of VY
        self.set_reg(reg_x, self.get_reg(reg_y));
        let carry = self.get_reg(reg_x) & 0x1;
        self.set_reg(reg_x, self.get_reg(reg_x) >> 1);
        self.set_carry(carry);
    }

    /// Opcode 8XY7
    /// Subtract the value of VX from VY and load result into VX (VX = VY - VX)
    /// then enable carry register if not borrowing
    fn sub_register_reversed(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);

        // Enable carry register if subtraction borrows
        let not_borrow = (self.get_reg(reg_y) >= self.get_reg(reg_x)) as u8;
        
        let result = self.get_reg(reg_y).wrapping_sub(self.get_reg(reg_x));
        self.set_reg(reg_x, result);
        
        self.set_carry(not_borrow);
    }

    /// Opcode 8XYE
    /// Set carry register to most significant bit of VX
    /// and shift VX one bit left
    fn shift_left(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        // Quirk set VX to value of VY
        self.set_reg(reg_x, self.get_reg(reg_y));
        let carry = (self.get_reg(reg_x) & 0x80) >> 7;
        self.set_reg(reg_x, self.get_reg(reg_x) << 1);
        self.set_carry(carry);
    }

    /// Opcode 9XY0
    /// Skip next instruction if VX != VY
    fn skip_register_not_equal(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        if self.get_reg(reg_x) != self.get_reg(reg_y) {
            self.pc += 2;
        }
    }

    /// Opcode ANNN
    /// Set I register to NNN
    fn load_i(&mut self, opcode: u16) {
        self.i_reg = opcode & 0x0FFF;
    }

    /// Opcode BNNN
    /// Jump to address at V0 + NNN
    fn jump_plus(&mut self, opcode: u16) {
        self.pc = self.get_reg(0) as u16 + (opcode & 0x0FFF);
    }

    /// Opcode CXNN
    /// Generate random number, R, from 0 to 255 and add R AND NN to VX
    fn random_and(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        let random: u8 = self.rng.random();
        self.v_reg[register] = random & number;
    }

    /// Opcode DXYN
    /// Draws N-byte (heigh of N pixels) on screen and enables
    /// carry register if there is collision
    fn draw_sprite(&mut self, opcode: u16, display: &mut Display) -> Result<()> {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let rows = opcode & 0x000F;

        // Check if sprite bounds are within valid RAM addresses
        if self.i_reg + rows > RAM_SIZE as u16 {
            return Err(Error::InvalidRamAddressError);
        }

        // Set x and y coords to VX and VY with wrapping for the starting coord
        let x_coord = self.get_reg(reg_x) % SCREEN_WIDTH as u8;
        let y_coord = self.get_reg(reg_y) % SCREEN_HEIGHT as u8;

        let sprite = self.memory.read_slice(self.i_reg, rows);

        // Draw sprite on screen
        let carry = display.draw(sprite, x_coord, y_coord);
        // Set carry register
        self.set_carry(carry);
        Ok(())
    }

    /// Opcode EX9E
    /// Skip next instruction if key VX is pressed down. Do not wait for input
    fn skip_if_keypress(&mut self, opcode: u16, input: &mut Input) {
        let register = (opcode & 0x0F00) >> 8;
        let key = self.get_reg(register);
        if input.check_key(key) {
            self.pc += 2;
        }
    }

    /// Opcode EXA1
    /// Skip next instruction if key VX is *not* pressed down. Do not wait for input
    fn skip_if_not_keypress(&mut self, opcode: u16, input: &mut Input) {
        let register = (opcode & 0x0F00) >> 8;
        let key = self.get_reg(register);
        if !input.check_key(key) {
            self.pc += 2;
        }
    }

    /// Opcode FX07
    /// Load value of delay timer into VX
    fn load_delay_timer(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let value = self.dt;
        self.set_reg(register, value);
    }

    /// Opcode FX0A
    /// Wait for key press and store key value in VX. If no key is pressed
    /// the PC is decremented to rerun opcode
    fn wait_for_keypress(&mut self, opcode: u16, input: &mut Input) {
        let register = (opcode & 0x0F00) >> 8;
        let key = match input.check_all_keys() {
            Some(key) => key,
            None => {
                self.pc -= 2;
                return;
            }
        };
        self.set_reg(register, key);
    }

    /// Opcode FX15
    /// Set delay timer to value of VX
    fn set_delay_timer(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        self.dt = self.get_reg(register);
    }

    /// Opcode FX18
    /// Set sound timer to value of VX
    fn set_sound_timer(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        self.st = self.get_reg(register);
    }

    /// Opcode FX1E
    /// Add I to VX and store in VX (I += VX)
    fn load_add_i(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        self.i_reg += self.get_reg(register) as u16;
    }

    /// Opcode FX29
    /// Set I register to the address of the character in font set
    /// corresponding to the value of VX
    fn find_character(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let key_value = self.get_reg(register);
        self.i_reg = FONTSET_ADDR + 5 * key_value as u16;
    }

    /// Opcode FX33
    /// Store binary-coded decimal conversion of number in VX to
    /// RAM adresses I register, I + 1 and I + 2
    fn store_bcd(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let number = self.get_reg(register);
        let hundreds = number / 100;
        let tens = (number / 10) % 10;
        let ones = number % 10;
        self.memory.write(self.i_reg, hundreds);
        self.memory.write(self.i_reg + 1, tens);
        self.memory.write(self.i_reg + 2, ones);
    }

    /// Opcode FX55
    /// Dump registers from V0 through VX to RAM starting at the
    /// address in I register
    fn dump_registers_to_ram(&mut self, opcode: u16) -> Result<()> {
        let register = (opcode & 0x0F00) >> 8;
        let reg_slice = &self.v_reg[0..=register as usize];
        self.memory.write_slice(reg_slice, self.i_reg)?;

        Ok(())
    }

    /// Opcode FX65
    /// Load values from memory starting form address in I register
    /// into V0 through VX
    fn load_registers_from_ram(&mut self, opcode: u16) {
        let register = (opcode & 0x0F00) >> 8;
        let address = self.i_reg;
        let memory_slice = self.memory.read_slice(address, register + 1);
        self.v_reg[..=register as usize].copy_from_slice(memory_slice);
    }
}
