pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    first_write: bool, // Whether the next write updates the X or Y scroll value
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            first_write: true, // write to X initially, then toggle to Y
        }
    }

    pub fn update(&mut self, value: u8) {
        if self.first_write {
            self.scroll_x = value;
        } else {
            self.scroll_y = value;
        }

        self.first_write = !self.first_write;
    }

    pub fn reset_latch(&mut self) {
        self.first_write = true;
    }
}
