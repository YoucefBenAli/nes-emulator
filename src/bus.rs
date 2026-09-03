use crate::memory::Memory;
use crate::ppu::PPU;
use crate::rom::Rom;
use std::cell::RefCell;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF; // CPU RAM 0x0000..0x1FFF

const PROGRAM_MEMORY_START: u16 = 0x8000;
const PROGRAM_MEMORY_END: u16 = 0xFFFF;

const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
pub struct Bus {
    cpu_ram: [u8; 2048], // 2^11 = 2048, on a real NES only 11 pins used for addressing cpu ram
    rom: Rom,
    ppu: RefCell<PPU>,
}

impl Bus {
    pub fn new(rom: Rom) -> Bus {
        let ppu = PPU::new(rom.character_rom.clone(), rom.mirroring);
        Bus {
            cpu_ram: [0; 2048],
            rom: rom,
            ppu: RefCell::new(ppu),
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
                let mirrored_address = PPU_REGISTERS + (address - PPU_REGISTERS) % 8;
                let mut ppu = self.ppu.borrow_mut();

                match mirrored_address {
                    0x2002 => ppu.read_status_register(),
                    0x2004 => ppu.read_oam_data_register(),
                    0x2007 => ppu.read_data(),
                    // Write-only PPU registers behave like an open bus when read.
                    // Until the bus tracks its last value, return zero.
                    _ => 0,
                }
            },
            _ => {
                // println!("Out of bounds ignoring read at {address}");
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
                let mirrored_address = PPU_REGISTERS + (address - PPU_REGISTERS) % 8;
                let ppu = self.ppu.get_mut();

                match mirrored_address {
                    0x2000 => ppu.write_to_control_register(value),
                    0x2001 => ppu.write_to_mask_register(value),
                    0x2002 => panic!("Attempt to write to read-only PPU register {:#06X}", address),
                    0x2003 => ppu.write_to_oam_address_register(value),
                    0x2004 => ppu.write_to_oam_data_register(value),
                    0x2005 => ppu.write_to_scroll_register(value),
                    0x2006 => ppu.write_to_address_register(value),
                    0x2007 => ppu.write_data(value),
                    _ => unreachable!(),
                }
            },
            0x8000..=0xFFFF => {
                panic!("Can't write to read only memory");
            },
            _ => {
                // println!("Out of bounds ignoring write at {address}");
            }
        }
    }
    
}
