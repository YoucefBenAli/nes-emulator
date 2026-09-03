use crate::rom::MirroringType;

mod registers;

use registers::address::AddressRegister;
use registers::control::ControlRegister;
use registers::mask::MaskRegister;
use registers::scroll::ScrollRegister;
use registers::status::StatusRegister;
pub struct PPU {
    pub chr_rom: Vec<u8>, // Character Rom for pixel patterns to draw tiles and sprites
    pub palette_table: [u8; 32], // colors to use, background and sprites (16 bytes background, 16 sprites)
    pub vram: [u8; 2048], // nametable ram (which tiles go where)
    pub oam_data: [u8; 256], // object attribute memory, basically the state of the sprites. Up to 64 sprites, 4 bytes per sprite (Y position, tile index, attributes, x position)
    oam_address: u8, // Address of the OAM byte accessed through OAMDATA

    pub mirroring: MirroringType,

    internal_buffer: u8, // PPU holds data in an internal buffer it immediatly returns during read requests and updates this value with the data in the address requested
    address_register: AddressRegister,
    control_register: ControlRegister,
    mask_register: MaskRegister,
    scroll_register: ScrollRegister,
    status_register: StatusRegister,
}

impl PPU {
   pub fn new(chr_rom: Vec<u8>, mirroring: MirroringType) -> Self {
       PPU {
           chr_rom: chr_rom,
           mirroring: mirroring,
           vram: [0; 2048],
           oam_data: [0; 64 * 4],
           oam_address: 0,
           palette_table: [0; 32],
           internal_buffer: 0,
           address_register: AddressRegister::new(),
           control_register: ControlRegister::new(),
           mask_register: MaskRegister::new(),
           scroll_register: ScrollRegister::new(),
           status_register: StatusRegister::new(),
       }
   }

   fn increment_vram_address(&mut self) {
        self.address_register.increment(self.control_register.increment_count());
   }

    fn mirror_palette_addr(&self, addr: u16) -> usize {
        // First mirror the entire $3F00-$3FFF range down
        // into the 32-byte palette range $3F00-$3F1F.
        let mut index = (addr - 0x3F00) % 0x20;

        // $3F10/$3F14/$3F18/$3F1C mirror
        // $3F00/$3F04/$3F08/$3F0C.
        if matches!(index, 0x10 | 0x14 | 0x18 | 0x1C) {
            index -= 0x10;
        }

        index as usize
    }

   pub fn read_data(&mut self) -> u8 {
        let address: u16 = self.address_register.get();
        self.increment_vram_address();

        match address {
            0..=0x1fff => {
                let result: u8 = self.internal_buffer;
                self.internal_buffer = self.chr_rom[address as usize];
                result
            },
            0x2000..=0x2fff => {
                let result: u8 = self.internal_buffer;
                self.internal_buffer = self.vram[self.mirror_vram_addr(address) as usize];
                result
            }
            0x3000..=0x3eff => panic!("Impossible to be here, address_register should mirror this down = {} ", address),
            0x3f00..=0x3fff =>
            {
                // 32 bytes of palette table
                self.palette_table[self.mirror_palette_addr(address)]
            }
            _ => panic!("Mirrored addresses, shouldnt be possible to read here {}", address),
        }
   }

   pub fn write_data(&mut self, value: u8) {
        let address: u16 = self.address_register.get();
        self.increment_vram_address();

        match address {
            0..=0x1fff => {
                panic!("Attempt to write to CHR ROM at address {}", address);
            },
            0x2000..=0x2fff => {
                let mirrored_address = self.mirror_vram_addr(address) as usize;
                self.vram[mirrored_address] = value;
            }
            0x3000..=0x3eff => panic!("Impossible to be here, address_register should mirror this down = {} ", address),
            0x3f00..=0x3fff => {
                // 32 bytes of palette table
                let mirrored_address = self.mirror_palette_addr(address);
                self.palette_table[mirrored_address] = value;
            }
            _ => panic!("Mirrored addresses, shouldnt be possible to write here {}", address),
        }
   }

   pub fn mirror_vram_addr(&self, mut addr: u16) -> u16 {
        addr &= 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        addr -= 0x2000; // 2000 extra bytes, reference start from 0
        let name_table = addr / 0x400; // 0x000-0x3FF nametable 0, 0x400-0x7FF nametable 1, 0x800-0xBFF nametable 2, 0xC00, 0xFFF nametable3
        match (&self.mirroring) {
            MirroringType::Vertical => {
                match name_table {
                    2 | 3 => addr - 0x800, // 0x800 to get to the nametable above you, 2 and 3 are bottom mirrors in vertical type
                    _ => addr
                }
            }
            MirroringType::Horizontal => {
                match name_table {
                    // Nametable 1 is mirrored to nametable 0 so just remove 0x400 to get to nametable 0, n
                    // Nametable 2 is the "real" second nametable but we need to map it to our vram which only goes up to 2048 bytes (0x800)
                    // So for nametable 2 we need to remove 0x400 to address that space
                    // Nametable 3 is the mirror for nametable 2 so we kind of need to remove 0x400 twice first to get to the real nametable 2, then to map it to the vector
                    1 | 2 => addr - 0x400,
                    3 => addr -0x800,
                    _ => addr, // nametable 0 is correct
                }
            }
            _ => addr, // just in case
        }
   }

   pub fn write_to_address_register(&mut self, value: u8) {
        self.address_register.update(value);
   }

   pub fn write_to_control_register(&mut self, value: u8) {
       self.control_register.update(value);
   }

   pub fn write_to_mask_register(&mut self, value: u8) {
       self.mask_register.update(value);
   }

   pub fn write_to_scroll_register(&mut self, value: u8) {
       self.scroll_register.update(value);
   }

   pub fn write_to_oam_address_register(&mut self, value: u8) {
       self.oam_address = value;
   }

   pub fn read_oam_data_register(&self) -> u8 {
       self.oam_data[self.oam_address as usize]
   }

   pub fn write_to_oam_data_register(&mut self, value: u8) {
       self.oam_data[self.oam_address as usize] = value;
       self.oam_address = self.oam_address.wrapping_add(1);
   }

   pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
       for value in data {
           self.write_to_oam_data_register(*value);
       }
   }

   pub fn read_status_register(&mut self) -> u8 {
       let status = self.status_register.get();

       // Reading PPUSTATUS clears vblank and resets the shared write latch used
       // by PPUSCROLL and PPUADDR.
       self.status_register.set_vblank(false);
       self.address_register.reset_latch();
       self.scroll_register.reset_latch();

       status
   }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_empty_ppu() -> PPU {
        PPU::new(vec![0; 8192], MirroringType::Horizontal)
    }

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_address_register(0x23);
        ppu.write_to_address_register(0x05);
        ppu.write_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_control_register(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_address_register(0x23);
        ppu.write_to_address_register(0x05);

        ppu.read_data(); // load into buffer
        assert_eq!(ppu.address_register.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_control_register(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_address_register(0x21);
        ppu.write_to_address_register(0xff);

        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_control_register(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_address_register(0x21);
        ppu.write_to_address_register(0xff);

        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_address_register(0x24);
        ppu.write_to_address_register(0x05);
        ppu.write_data(0x66); // write to a

        ppu.write_to_address_register(0x28);
        ppu.write_to_address_register(0x05);
        ppu.write_data(0x77); // write to B

        ppu.write_to_address_register(0x20);
        ppu.write_to_address_register(0x05);
        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66); // read from A

        ppu.write_to_address_register(0x2c);
        ppu.write_to_address_register(0x05);
        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x77); // read from b
    }

    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = PPU::new(vec![0; 2048], MirroringType::Vertical);

        ppu.write_to_address_register(0x20);
        ppu.write_to_address_register(0x05);
        ppu.write_data(0x66); // write to A

        ppu.write_to_address_register(0x2c);
        ppu.write_to_address_register(0x05);
        ppu.write_data(0x77); // write to b

        ppu.write_to_address_register(0x28);
        ppu.write_to_address_register(0x05);
        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66); // read from a

        ppu.write_to_address_register(0x24);
        ppu.write_to_address_register(0x05);
        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x77); // read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = new_empty_ppu();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_address_register(0x21);
        ppu.write_to_address_register(0x23);
        ppu.write_to_address_register(0x05);

        ppu.read_data(); // load into buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status_register();

        ppu.write_to_address_register(0x23);
        ppu.write_to_address_register(0x05);

        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_control_register(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_address_register(0x63); // 0x6305 -> 0x2305
        ppu.write_to_address_register(0x05);

        ppu.read_data(); // load into buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = new_empty_ppu();
        ppu.status_register.set_vblank(true);

        let status = ppu.read_status_register();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status_register.get() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = new_empty_ppu();
        ppu.write_to_oam_address_register(0x10);
        ppu.write_to_oam_data_register(0x66);
        ppu.write_to_oam_data_register(0x77);

        ppu.write_to_oam_address_register(0x10);
        assert_eq!(ppu.read_oam_data_register(), 0x66);

        ppu.write_to_oam_address_register(0x11);
        assert_eq!(ppu.read_oam_data_register(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = new_empty_ppu();
        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.write_to_oam_address_register(0x10);
        ppu.write_oam_dma(&data);

        ppu.write_to_oam_address_register(0x0f); // wrap around
        assert_eq!(ppu.read_oam_data_register(), 0x88);

        ppu.write_to_oam_address_register(0x10);
        assert_eq!(ppu.read_oam_data_register(), 0x77);

        ppu.write_to_oam_address_register(0x11);
        assert_eq!(ppu.read_oam_data_register(), 0x66);
    }
}
