pub struct AddressRegister {
   data: u16,
   first_byte: bool, // Whether we're pointing to the first byte or second byte
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            data: 0,
            first_byte: true, // big endian, write to first byte initially then toggle
        }
    }

    pub fn get(&self) -> u16 {
        self.data
    }

    // this function simulates writes to 0x2007 data register by CPU
    pub fn update(&mut self, value: u8) {
        if self.first_byte {
            // Update first 8 bits only
            self.data = (self.data & 0x00FF) | ((value as u16) << 8);
        } else {
            // Update last 8 bits only
            self.data = (self.data & 0xFF00) | (value as u16);
        }

        //PPU only has as 14 bit address bus so anything above this needs to mirror down to the same address
        //so we can simply & the first 14 bits
        self.data = self.data & 0b0011_1111_1111_1111;
        self.first_byte = !self.first_byte;
    }
    
    pub fn increment(&mut self, increment: u8) {
        //Increment can be 1 or 32 depending on if we're going to next column or next row in the nametable
        // nametable is set out in a 32 column tile grid
        self.data = self.data.wrapping_add(increment as u16) & 0b0011_1111_1111_1111;
    }

    pub fn reset_latch(&mut self) {
        self.first_byte = true;
    }
}
