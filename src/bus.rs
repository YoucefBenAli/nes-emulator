use crate::memory::Memory;
use crate::rom::Rom;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF; // CPU RAM 0x0000..0x1FFF

const PROGRAM_MEMORY_START: u16 = 0x8000;
const PROGRAM_MEMORY_END: u16 = 0xFFFF;

const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
pub struct Bus {
    cpu_ram: [u8; 2048], // 2^11 = 2048, on a real NES only 11 pins used for addressing cpu ram
    rom: Rom,
}

impl Bus {
    pub fn new(rom: Rom) -> Bus {
        Bus {
            cpu_ram: [0; 2048],
            rom: rom,
        }
    }

    fn read_program_memory(&self, mut address: u16) -> u8 {
        // Program memory can be either 16kb long or 32kb long, if its only 16 we need to mirror anything higher than 16kb to its equivalent
        if (self.rom.program_rom.len() == 0x4000 && address >= 0x4000) {
            address %= 0x4000;
        }
        self.rom.program_rom[address as usize]
    }
}

impl Memory for Bus {
    fn mem_read(&self, address: u16) -> u8 {
        match address {
            RAM..=RAM_MIRRORS_END => {
                // Since there are only 11 pins on the physical hardware we only care about the 11 least significant bits
                let trimmed_address = address & 0b_0000_0111_1111_1111;
                self.cpu_ram[trimmed_address as usize]
            },
            PROGRAM_MEMORY_START..=PROGRAM_MEMORY_END => {
                self.read_program_memory(address - 0x8000)
            },
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                0
            },
            _ => {
                println!("Out of bounds ignoring read at {address}");
                0
            }
        }
    }

    fn mem_write(&mut self, address: u16, value:u8) {
        match address {
            RAM..=RAM_MIRRORS_END => {
                // Since there are only 11 pins on the physical hardware we only care about the 11 least significant bits
                let trimmed_address = address & 0b_0000_0111_1111_1111;
                self.cpu_ram[trimmed_address as usize] = value
            },
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
               0;
            },
            0x8000..=0xFFFF => {
                panic!("Can't write to read only memory");
            },
            _ => {
                println!("Out of bounds ignoring read at {address}");
            }
        }
    }
    
}