pub struct MaskRegister {
    state: u8, // bit flags: BGRs_bMmG
    // BGR: Emphasize blue, green, and red respectively
    // s: Show sprites
    // b: Show background
    // M: Show sprites in the leftmost 8 pixels
    // m: Show background in the leftmost 8 pixels
    // G: Greyscale, 0 -> normal colour, 1 -> greyscale
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister {state: 0}
    }

    pub fn update(&mut self, bits: u8) {
        self.state = bits;
    }

    pub fn is_greyscale(&self) -> bool {
        self.state & 0b0000_0001 != 0
    }

    pub fn show_background_leftmost(&self) -> bool {
        self.state & 0b0000_0010 != 0
    }

    pub fn show_sprites_leftmost(&self) -> bool {
        self.state & 0b0000_0100 != 0
    }

    pub fn show_background(&self) -> bool {
        self.state & 0b0000_1000 != 0
    }

    pub fn show_sprites(&self) -> bool {
        self.state & 0b0001_0000 != 0
    }

    pub fn emphasize_red(&self) -> bool {
        self.state & 0b0010_0000 != 0
    }

    pub fn emphasize_green(&self) -> bool {
        self.state & 0b0100_0000 != 0
    }

    pub fn emphasize_blue(&self) -> bool {
        self.state & 0b1000_0000 != 0
    }
}
