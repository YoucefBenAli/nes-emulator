pub trait Memory {
    fn mem_read(&self, address: u16) -> u8;
    fn mem_write(&mut self, address: u16, value:u8);

    fn mem_read_u16(&self, address: u16) -> u16 {
        let low_byte: u16 = self.mem_read(address) as u16 & 0b0000_0000_1111_1111;
        let high_byte: u16 = (self.mem_read(address.wrapping_add(1)) as u16) << 8;

        high_byte | low_byte
    }

    fn mem_write_u16(&mut self, address: u16, value: u16) {
        let high_byte: u8 = (value >> 8) as u8;
        let low_byte: u8 = (value & 0b0000_0000_1111_1111) as u8;

        self.mem_write(address, low_byte);
        self.mem_write(address.wrapping_add(1), high_byte);
    }
}