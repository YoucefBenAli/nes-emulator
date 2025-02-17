use crate::memory::Memory;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF; // CPU RAM 0x0000..0x1FFF

pub struct Bus {
    cpu_ram: [u8; 2048], // 2^11 = 2048, on a real NES only 11 pins used for addressing cpu ram
    program_memory: [u8; 0x2000], // temporary just to get unit tests working
    program_counter_initial_value: [u8; 2], //temporary just to get unit tests working
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cpu_ram: [0; 2048],
            program_memory: [0; 0x2000],
            program_counter_initial_value: [0; 2],
        }
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

            0x8000..=0x9FFF => {
                //temporary measure to allow unit tests to pass
                self.program_memory[(address - 0x8000) as usize]
            },

            0xFFFC..=0xFFFD => {
                self.program_counter_initial_value[(address-0xFFFC) as usize]
            }
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

            0x8000..=0x8FFF => {
                //temporary measure to allow unit tests to pass
                self.program_memory[(address - 0x8000) as usize] = value // need to subtract 0x8000 to keep it in bounds of the program memory vector
            },

            0xFFFC..=0xFFFD => {
                self.program_counter_initial_value[(address-0xFFFC) as usize] = value
            }
            _ => {
                println!("Out of bounds ignoring read at {address}");
            }
        }
    }
    
}