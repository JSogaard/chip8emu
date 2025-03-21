use crate::errors::Error;
use crate::errors::Result;

pub const RAM_SIZE: usize = 4096;
pub const START_ADDR: u16 = 0x200;
pub const MAX_ROM_SIZE: usize = RAM_SIZE - START_ADDR as usize;

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

#[derive(Debug)]
pub struct Memory {
    ram: [u8; RAM_SIZE],
    rom_loaded: bool,
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = Self {
            ram: [0; RAM_SIZE],
            rom_loaded: false,
        };

        // Copying font set into ram from address 0x50 (80)
        // Get target location in RAM as slice and copy font set to it
        memory.ram[FONTSET_ADDR as usize..(FONTSET_ADDR as usize + FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        memory
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() <= MAX_ROM_SIZE {
            self.ram[START_ADDR as usize..].copy_from_slice(rom);
        } else {
            return Err(Error::InvalidRomSizeError);
        }
        self.rom_loaded = true;

        Ok(())
    }

    pub fn rom_loaded(&self) -> bool {
        self.rom_loaded
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }

    pub fn read_slice(&self, address: u16, length: u16) -> &[u8] {
        let address = address as usize;
        let length = length as usize;
        &self.ram[address..address + length]
    }

    pub fn write_slice(&mut self, slice: &[u8], address: u16) -> Result<()> {
        let address = address as usize;
        let length = slice.len();
        // Check if memory addresses are valid
        if address + length > self.ram.len() {
            return Err(Error::InvalidRamAddressError);
        }
        self.ram[address..address + length].copy_from_slice(slice);

        Ok(())
    }
    
    pub fn reset(&mut self) {
        self.ram = [0; RAM_SIZE];
        self.rom_loaded = false;
    }
}
