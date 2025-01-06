mod AddressingModes;
mod OpCodes;

use AddressingModes::AddressingMode;
use OpCodes::{OpCode, OPCODES_MAP};

struct CPU {
    // TODO: need to set these private and make getters/setters
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub state: u8, // bit flags: NV-BDIZC where N=negative V=overflow, B=break, D=decimal mode, I=interrupt, Z=zero, C=carry
    pub program_counter: u16,
    memory: [u8; 0xFFFF] //64kb long array
}

impl CPU {
    fn new() -> CPU {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            state: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }
    
    //-- Memory read and write methods

    pub fn mem_read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn mem_write(&mut self, address: u16, value:u8) {
        self.memory[address as usize] = value;
    }

    pub fn mem_read_u16(&self, address: u16) -> u16 {
        let low_byte: u16 = self.mem_read(address) as u16 & 0b0000_0000_1111_1111;
        let high_byte: u16 = (self.mem_read(address.wrapping_add(1)) as u16) << 8;

        high_byte | low_byte
    }

    pub fn mem_write_u16(&mut self, address: u16, value: u16) {
        let high_byte: u8 = (value >> 8) as u8;
        let low_byte: u8 = (value & 0b0000_0000_1111_1111) as u8;

        self.mem_write(address, low_byte);
        self.mem_write(address.wrapping_add(1), high_byte);
    }

    //-- Core CPU methods

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.state = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // Load the program starting at 0x8000 since that's where program ROM is allocated according to NES specs
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn run(&mut self) {
        
        loop {
            let curr_instruction: u8 = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let initial_program_counter: u16 = self.program_counter;

            let opcode: &OpCode = OPCODES_MAP.get(&curr_instruction).expect("Instruction: {curr_instruction} not found");

            match curr_instruction {
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(opcode.get_mode()),
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(opcode.get_mode()),
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(opcode.get_mode()),
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(opcode.get_mode()),
                0x86 | 0x96 | 0x8E => self.stx(opcode.get_mode()),
                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => { // BRK command
                    break;
                }
                _ => todo!("Instruction {curr_instruction} hasn't been implemented yet or is invalid")
            }

            if initial_program_counter == self.program_counter {
                self.program_counter += (opcode.get_num_bytes() - 1) as u16;
            }
        }
        return;
    }

    //-- CPU Instructions

    // Reference: https://www.nesdev.org/obelisk-6502-guide/reference.html

    fn lda(&mut self, mode: &AddressingMode) {
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_a = param;
                
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr: u16 = mode.get_operand_address(self);
        self.mem_write(addr, self.reg_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        // Missing unit tests
        let addr: u16 = mode.get_operand_address(self);
        self.mem_write(addr, self.reg_x);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        // Missing unit tests
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_x = param;
        
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        // Missing unit tests
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_y = param;
        
        self.set_zero_and_negative_flag(self.reg_y);
    }

    //-- Helper methods

    /// Sets the zero flag if value is 0 and negative flag if value is negative
    fn set_zero_and_negative_flag(&mut self, value: u8) {
        self.set_zero_flag(value==0);
        self.set_negative_flag(Self::is_register_negative(value));

    }

    fn set_zero_flag(&mut self, is_zero: bool) {
        if is_zero {
            self.state |= 0b0000_0010;
        } else {
            self.state &= 0b1111_1101;
        }
    }

    fn set_negative_flag(&mut self, is_negative: bool) {
        if is_negative {
            self.state |= 0b1000_0000;
        } else {
            self.state &= 0b0111_1111;
        }
    }

    fn is_register_negative(register_value: u8) -> bool {
        //the negative flag is set if bit 7 (last bit) is 1
        register_value & 0b1000_0000 != 0
    }

    fn get_and_increment_program_counter(&mut self) -> u8 {
        let param: u8 = self.mem_read(self.program_counter);
        self.program_counter+=1;
        return param;
    }

    fn is_zero_flag_set(&self) -> bool {
        self.state & 0b0000_0010 != 0
    }

    fn is_negative_flag_set(&self) -> bool {
        self.state & 0b1000_0000 != 0
    }

}
fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod test {
    use super::*;
   
    // ---------- LDA tests
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa5_lda_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x09; // Assign value 9 to memory location 0x05
        cpu.load_and_run(vec![0xa5, 0x05, 0x00]); // Load value at memory location 0x05 using lda
        assert_eq!(cpu.reg_a, 0x09);
    }

    #[test]
    fn test_0xb5_lda_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x09] = 0x07; // Assign value 7 to memory location 0x09 (0x05 + 0x04)
        cpu.load_and_run(vec![0xa2, 0x05, 0xb5, 0x04, 0x00]); // Load 0x05 into reg_x, then load A with the value stored at 0x04 + reg_x (0x04+0x05=0x09) which has value 7
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x07;
        cpu.load_and_run(vec![0xad, 0x00, 0x10, 0x00]); // LDA $1000 (little endian so its bytes 0x00 and then 0x10)
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa2, 0x05, 0xbd, 0x00, 0x10, 0x00]); //Load 0x05 into reg_x then LDA $1000,X (0x1000+0x0005=0x1005)
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa0, 0x05, 0xb9, 0x00, 0x10, 0x00]); //Load 0x05 into reg_y then LDA $1000,Y (0x1000+0x0005=0x1005)
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_indirect_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        // Little endian storage of address 1005 (least significant [0x05] first then most [0x10])
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa2, 0x05, 0xa1, 0x05, 0x00]); //Load 0x05 into reg_x then LDA ($05,X) (0x05 + 0x05 = 0x0a => address referenced at 0x0a = 0x1005)
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_indirect_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x100a] = 0x07;
        // Little endian storage of address 1005 (least significant [0x05] first then most [0x10])
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa0, 0x05, 0xb1, 0x0a, 0x00]); //Load 0x05 into reg_y then LDA ($0a),Y (addr referenced at 0x0a => 0x1005, add x (0x05) => 0x100a)
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.state & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]); // Transfer value 0x80 into accumulator which corresponds to 0b1000_0000
        assert!(cpu.is_negative_flag_set());
    }
    

    // ---------- TAX tests
    #[test]
    fn test_0xaa_tax_a_is_zero() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x00, 0xaa, 0x00]; // Transfer value 0 into accumulator and TAX

        cpu.load_and_run(program);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xaa_tax_a_is_negative() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x80, 0xaa, 0x00]; // Transfer value 0x80 into accumulator which corresponds to 0b1000_0000 and TAX

        cpu.load_and_run(program);
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    #[test]
    fn test_0xaa_tax_neither_negative_or_zero() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x20, 0xaa, 0x00]; // Transfer value 0x20 (decimal: 32) into accumulator and TAX

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    // ---------- INX tests
    #[test]
    fn test_0xe8_inx_normal_increment() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x20, 0xaa, 0xe8, 0x00]; // Transfer value 0x20 (decimal: 32) into accumulator and TAX and the increment

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.reg_x==33);
    }
    #[test]
    fn test_0xe8_inx_increment_into_negative() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x7F, 0xaa, 0xe8, 0x00]; // Transfer value 0x7F (decimal: 127) into accumulator and TAX and the increment which will go negative

        cpu.load_and_run(program);
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }
    #[test]
    fn test_0xe8_inx_increment_into_zero() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0xFF, 0xaa, 0xe8, 0x00]; // Transfer value 0xFF (decimal: 255) into accumulator and TAX and the increment which will go zero

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_x, 0);
    }

    // ----------- Extra tests from the book
    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
  
        assert_eq!(cpu.reg_x, 0xc1)
    }

}