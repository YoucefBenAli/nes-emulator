pub struct ControlRegister {
   state: u8, //bit flags: VPHB_SINN
   // V: Generate NMI (Non maskable interrupt), 0 off, 1 on
   // P: master slave select, 0 read from ext, 1 write to ext, prob gonna remain unused
   // H: Sprite size
   // B: 0 -> background patterns from $0000, 1 -> $1000
   // S: 0 -> sprite patterns come from $0000, 1 -> $1000
   // I: Increment, 0-> increment vram by 1, 1-> increment vram by 32
   // NN: Base nametable 00->$2000 01->$2400 10->$2800 11->$2C00
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister {state: 0}
    }

    pub fn update(&mut self, bits: u8) {
        self.state = bits;
    }

    pub fn increment_count(&self) -> u8 {
       if (self.state >> 2 & 1) == 1 {
           32
       } else {
           1
       }
   }
}