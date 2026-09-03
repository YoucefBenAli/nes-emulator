pub struct StatusRegister {
    state: u8, // bit flags: VSOx_xxxx
    // V: Vertical blank has started
    // S: Sprite zero hit
    // O: Sprite overflow
    // x: Unused bits
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister {state: 0}
    }

    pub fn get(&self) -> u8 {
        self.state
    }

    pub fn is_in_vblank(&self) -> bool {
        self.state & 0b1000_0000 != 0
    }

    pub fn set_vblank(&mut self, value: bool) {
        self.set_bit(7, value);
    }

    pub fn set_sprite_zero_hit(&mut self, value: bool) {
        self.set_bit(6, value);
    }

    pub fn set_sprite_overflow(&mut self, value: bool) {
        self.set_bit(5, value);
    }

    fn set_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.state |= 1 << bit;
        } else {
            self.state &= !(1 << bit);
        }
    }
}
