const PRG_ROM_BANK_SIZE_IN_BYTES: usize = 16384; //16kb = 16*1024
const CHR_ROM_BANK_SIZE_IN_BYTES: usize = 8192; //8kb = 8*1024


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MirroringType {
   Vertical,
   Horizontal,
   FourScreen,
}

pub struct Rom {
   pub program_rom: Vec<u8>,
   pub character_rom: Vec<u8>,
   pub mapper: u8,
   pub mirroring: MirroringType,
}

impl Rom {
    pub fn new(rom_dump: &Vec<u8>) -> Result<Rom, String> {
        // Header is always 16 bytes, so thats the minimum length
        if rom_dump.len() < 16 {
            return Err("Cartridge_data/ROM dump is invalid".to_string());
        }

        // First four bits need to match 4E 45 53 1A
        if rom_dump[0] != 0x4E || rom_dump[1] != 0x45 || rom_dump[2] != 0x53 || rom_dump[3] != 0x1A {
            return Err("ROM Dump is not .NES file".to_string());
        }

        let num_prg_rom_banks: usize = rom_dump[4] as usize;
        let num_prg_chr_banks: usize = rom_dump[5] as usize;

        let control_byte_1: u8 = rom_dump[6];
        let control_byte_2: u8 = rom_dump[7];

        // iNes format check
        if (Rom::read_nth_bit(control_byte_2, 3) || Rom::read_nth_bit(control_byte_2, 2)) {
            return Err("Only support iNES format 1.0".to_string());
        }

        let mapper:u8 = (control_byte_1 & 0b1111_0000) | (control_byte_2 >> 4);

        let has_battery_packed_ram: bool = Rom::read_nth_bit(control_byte_1, 1);
        let has_trainer: bool = Rom::read_nth_bit(control_byte_1, 2);

        let has_four_screen_vram: bool = Rom::read_nth_bit(control_byte_1, 3);
        let vertical_mirroring:bool = Rom::read_nth_bit(control_byte_1, 0);

        let mirroring: MirroringType = if has_four_screen_vram {
            MirroringType::FourScreen
        } else if vertical_mirroring {
            MirroringType::Vertical
        } else {
            MirroringType::Horizontal
        };

        let mut prg_rom_start: usize = 16; // Directly after header
        if has_trainer {prg_rom_start += 512};

        let chr_rom_start: usize = prg_rom_start + num_prg_rom_banks * PRG_ROM_BANK_SIZE_IN_BYTES;


        Ok(
            Rom {
                program_rom: rom_dump[prg_rom_start..(prg_rom_start + num_prg_rom_banks * PRG_ROM_BANK_SIZE_IN_BYTES)].to_vec(),
                character_rom: rom_dump[chr_rom_start..(chr_rom_start + num_prg_chr_banks * CHR_ROM_BANK_SIZE_IN_BYTES)].to_vec(),
                mapper,
                mirroring,
            }
        )
    }

    // Helper method to read n'th bit where the LSB is bit 0
    fn read_nth_bit(source: u8, bit_number: u8) -> bool {
        // not handling the case where bit_number is above or equal to 8 because who cares
        source & (1 << bit_number) != 0
    }
}