mod AddressingModes;
mod OpCodes;

use std::ops::Add;

use AddressingModes::AddressingMode;
use OpCodes::{Mnemonic, OpCode, OPCODES_MAP};
use OpCodes::Mnemonic::*;

struct CPU {
    // TODO: need to set these private and make getters/setters
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub state: u8, // bit flags: NV-BDIZC where N=negative V=overflow, B=break, D=decimal mode, I=interrupt, Z=zero, C=carry
    pub program_counter: u16,
    pub stack_ptr: u8,
    memory: [u8; 0xFFFF] //64kb long array
}

impl CPU {
    fn new() -> CPU {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            state: 0b0010_0000, // 5th bit always set
            program_counter: 0,
            stack_ptr: 0,
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
        self.state = 0b0010_0000; // 5th bit always set
        self.stack_ptr = 0xFF; // In 6502 the stack pointer always starts at 0x01ff

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

            let opcode: &OpCode = OPCODES_MAP.get(&curr_instruction).expect(&format!("Instruction: {curr_instruction} not found"));
            let mode: &AddressingMode = opcode.get_mode();
            let instruction: Mnemonic = opcode.get_instruction();

            match instruction {
                ADC => self.adc(mode),
                AND => self.and(mode),
                LDA => self.lda(mode),
                ASL => self.asl(mode),
                LSR => self.lsr(mode),
                LDX => self.ldx(mode),
                LDY => self.ldy(mode),
                STA => self.sta(mode),
                STX => self.stx(mode),
                STY => self.sty(mode),
                BIT => self.bit(mode),
                BCS => self.bcs(mode),
                BCC => self.bcc(mode),
                BEQ => self.beq(mode),
                BMI => self.bmi(mode),
                BNE => self.bne(mode),
                BPL => self.bpl(mode),
                BVC => self.bvc(mode),
                BVS => self.bvs(mode),
                CMP => self.cmp(mode),
                CPX => self.cpx(mode),
                CPY => self.cpy(mode),
                EOR => self.eor(mode),
                ORA => self.ora(mode),
                DEC => self.dec(mode),
                INC => self.inc(mode),
                JMP => self.jmp(mode),
                JSR => self.jsr(mode),
                ROL => self.rol(mode),
                ROR => self.ror(mode),
                PHP => self.php(),
                PLP => self.plp(),
                PLA => self.pla(),
                PHA => self.pha(),
                DEX => self.dex(),
                DEY => self.dey(),
                TAX => self.tax(),
                INX => self.inx(),
                INY => self.iny(),
                CLC => self.clc(),
                CLD => self.cld(),
                CLI => self.cli(),
                CLV => self.clv(),
                SEC => self.sec(),
                SED => self.sed(),
                SEI => self.sei(),
                NOP => self.nop(),

                BRK => {
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

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_a = param;
                
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr: u16 = mode.get_operand_address(self);
        self.mem_write(addr, self.reg_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr: u16 = mode.get_operand_address(self);
        self.mem_write(addr, self.reg_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr: u16 = mode.get_operand_address(self);
        self.mem_write(addr, self.reg_y);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_x = param;
        
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let param = self.mem_read(mode.get_operand_address(&self));
        self.reg_y = param;
        
        self.set_zero_and_negative_flag(self.reg_y);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        let mut sum_as_u16: u16 = self.reg_a as u16 + param as u16;
        if self.is_carry_flag_set() {
            sum_as_u16 += 1;
        }

        let sum_as_u8: u8 = sum_as_u16.to_be_bytes()[1]; // Get last 8 bits

        self.set_carry_flag(sum_as_u16 > 0b1111_1111);
        self.set_overflow_flag(
            ((param ^ sum_as_u8) & (self.reg_a ^ sum_as_u8) & 0x80) != 0
        );
        self.set_zero_and_negative_flag(sum_as_u8);
        
        self.reg_a = sum_as_u8;

    }

    fn and(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.reg_a = self.reg_a & param;

        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let new_val: u8 = param << 1;

        self.set_carry_flag((param & 0b1000_0000) != 0);

        self.set_negative_flag((new_val & 0b1000_0000) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
        
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let new_val: u8 = param >> 1;

        self.set_carry_flag((param & 0b0000_0001) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }

        
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        if !self.is_carry_flag_set() {
            self.branch(mode);
        }
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        if self.is_carry_flag_set() {
            self.branch(mode);
        }
    }

    fn beq(&mut self, mode: &AddressingMode) {
        if self.is_zero_flag_set() {
            self.branch(mode);
        }
    }

    fn bne(&mut self, mode: &AddressingMode) {
        if !self.is_zero_flag_set() {
            self.branch(mode);
        }
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        if self.is_negative_flag_set() {
            self.branch(mode);
        }
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        if !self.is_negative_flag_set() {
            self.branch(mode);
        }
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        if !self.is_overflow_flag_set() {
            self.branch(mode);
        }
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        if self.is_overflow_flag_set() {
            self.branch(mode);
        }
    }


    fn bit(&mut self, mode: &AddressingMode) {
        let mut param: u8 = self.mem_read(mode.get_operand_address(&self));

        let result = self.reg_a & param;
        
        // Z 	Zero Flag 	    Set if the result if the AND is zero
        // V 	Overflow Flag 	Set to bit 6 of the memory value
        // N 	Negative Flag 	Set to bit 7 of the memory value
        self.set_zero_flag(result==0);
        self.set_overflow_flag(Self::check_bit_set(param, 6));
        self.set_negative_flag(Self::check_bit_set(param, 7));
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.compare(self.reg_a, param);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.compare(self.reg_x, param);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.compare(self.reg_y, param);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let mut param: u8 = self.mem_read(mode.get_operand_address(&self));
        
        param = param.wrapping_sub(1);

        self.set_zero_and_negative_flag(param);
        self.mem_write(mode.get_operand_address(&self), param);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.reg_a = self.reg_a ^ param;
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));

        self.reg_a = self.reg_a | param;
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let mut param: u8 = self.mem_read(mode.get_operand_address(&self));

        param = param.wrapping_add(1);

        self.set_zero_and_negative_flag(param);
        self.mem_write(mode.get_operand_address(&self), param);
    }

    fn jmp (&mut self, mode: &AddressingMode) {
        let param: u16 = mode.get_operand_address(&self);

        self.program_counter = param;
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        let param: u16 = mode.get_operand_address(&self);

        // The JSR instruction pushes the address (minus one) of the return point on to the stack,
        // but we also need to add the 2 bytes read from the absoltue address
        self.push_to_stack_u16(self.program_counter +2 -1); 
        self.program_counter = param;
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let mut new_val: u8 = param << 1;

        if self.is_carry_flag_set() {
            new_val = new_val | 0b_0000_0001;
        }

        self.set_carry_flag((param & 0b1000_0000) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let mut new_val: u8 = param >> 1;

        if self.is_carry_flag_set() {
            new_val = new_val | 0b_1000_0000;
        }

        self.set_carry_flag((param & 0b0000_0001) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
    }

    fn php(&mut self) {
        // Need to set the 5th bit (the B flag) since the PHP and BRK instructions set it
        // Also need to set the 6th bit since its always pushed as 1
        // https://www.nesdev.org/wiki/Status_flags#The_B_flag
        let processor_flags: u8 = self.state | 0b0011_0000;
        self.push_to_stack_u8(processor_flags);
    }

    fn plp(&mut self) {
        self.state = self.pull_from_stack_u8();
    }

    fn pla(&mut self) {
        self.reg_a = self.pull_from_stack_u8();
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn pha(&mut self) {
        self.push_to_stack_u8(self.reg_a); 
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn iny(&mut self) {
        self.reg_y = self.reg_y.wrapping_add(1);
        self.set_zero_and_negative_flag(self.reg_y);
    }

    fn dex(&mut self) {
        
        let result: u8 = self.reg_x.wrapping_sub(1);
        
        self.set_zero_and_negative_flag(result);
        self.reg_x = result;
    }

    fn dey(&mut self) {
        
        let result: u8 = self.reg_y.wrapping_sub(1);
        
        self.set_zero_and_negative_flag(result);
        self.reg_y = result;
    }

    fn clc(&mut self) {
        self.set_carry_flag(false);
    }

    fn cld(&mut self) {
        self.set_decimal_flag(false);
    }

    fn cli(&mut self) {
        self.set_interrupt_flag(false);
    }

    fn clv(&mut self) {
        self.set_overflow_flag(false);
    }

    fn sec(&mut self) {
        self.set_carry_flag(true);
    }

    fn sed(&mut self) {
        self.set_decimal_flag(true);
    }

    fn sei(&mut self) {
        self.set_interrupt_flag(true);
    }

    fn nop(&self) {
        return;
    }


    //-- Helper methods

    fn push_to_stack_u16(&mut self, value:u16) {

        let value_low: u8 = (value & 0x00FF) as u8;
        let value_high: u8 = ((value & 0xFF00) >> 8) as u8;

        // High bit is pushed first then low bit is pushed
        self.push_to_stack_u8(value_high);
        self.push_to_stack_u8(value_low);
    }

    fn push_to_stack_u8(&mut self, value: u8) {
        let addr: u16 = (0x0100 as u16) | (self.stack_ptr as u16);
        self.mem_write(addr, value);
        self.stack_ptr = self.stack_ptr.wrapping_sub(1);
    }

    fn pull_from_stack_u8(&mut self) -> u8 {
        let addr: u16 = (0x0100 as u16) | (self.stack_ptr.wrapping_add(1) as u16);
        let value: u8 = self.mem_read(addr);
        self.stack_ptr = self.stack_ptr.wrapping_add(1);
        return value;
    }

    fn compare(&mut self, a: u8, b:u8) {
        
        self.set_carry_flag(a>=b);

        let result: u8 = a.wrapping_sub(b); // Z,C,N = A-B
        self.set_zero_and_negative_flag(result);

    }

    /// Returns true if the bit_to_check bit is set in param where bit 7 is the most significant bit and bit 0 is the least significant bit
    fn check_bit_set(param: u8, bit_to_check: u8) -> bool {
        if (bit_to_check >= 8) {
            panic!("Cant check above bit 7");
        }

        (param & (1 << bit_to_check)) != 0
    }

    fn branch(&mut self, mode: &AddressingMode) {
        let param = self.mem_read(mode.get_operand_address(&self));
        self.program_counter += 1; // Reading the byte containing the param
        self.program_counter += param as u16;
    }

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

    fn set_carry_flag(&mut self, carry: bool) {
        if carry {
            self.state |= 0b0000_0001;
        } else {
            self.state &= 0b1111_1110;
        }
    }

    fn set_overflow_flag(&mut self, overflow: bool) {
        if overflow {
            self.state |= 0b0100_0000;
        } else {
            self.state &= 0b1011_1111;
        }
    }

    fn set_break_flag(&mut self, break_flag: bool) {
        if break_flag {
            self.state |= 0b0001_0000;
        } else {
            self.state &= 0b1110_1111;
        }
    }

    fn set_decimal_flag(&mut self, decimal: bool) {
        if decimal {
            self.state |= 0b0000_1000;
        } else {
            self.state &= 0b1111_0111;
        }
    }

    fn set_interrupt_flag(&mut self, interrupt: bool) {
        if interrupt {
            self.state |= 0b0000_0100;
        } else {
            self.state &= 0b1111_1011;
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

    fn is_carry_flag_set(&self) -> bool {
        self.state & 0b0000_0001 != 0
    }

    fn is_overflow_flag_set(&self) -> bool {
        self.state & 0b0100_0000 != 0
    }

    fn is_break_flag_set(&self) -> bool {
        self.state & 0b0001_0000 != 0
    }

    fn is_decimal_flag_set(&self) -> bool {
        self.state & 0b0000_1000 != 0
    }

    fn is_interrupt_flag_set(&self) -> bool {
        self.state & 0b0000_0100 != 0
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
    
    // ---------- LDX tests

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa6_ldx_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x09; // Assign value 9 to memory location 0x05
        cpu.load_and_run(vec![0xa6, 0x05, 0x00]); // Load value at memory location 0x05 using lda
        assert_eq!(cpu.reg_x, 0x09);
    }

    #[test]
    fn test_0xb6_ldx_zero_page_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x09] = 0x07; // Assign value 7 to memory location 0x09 (0x05 + 0x04)
        cpu.load_and_run(vec![0xa0, 0x05, 0xb6, 0x04, 0x00]); // Load 0x05 into reg_y, then load A with the value stored at 0x04 + reg_y (0x04+0x05=0x09) which has value 7
        assert_eq!(cpu.reg_x, 0x07);
    }

    #[test]
    fn test_0xae_ldx_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x07;
        cpu.load_and_run(vec![0xae, 0x00, 0x10, 0x00]); // LDX $1000 (little endian so its bytes 0x00 and then 0x10)
        assert_eq!(cpu.reg_x, 0x07);
    }

    #[test]
    fn test_0xbe_ldx_absolute_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa0, 0x05, 0xbe, 0x00, 0x10, 0x00]); //Load 0x05 into reg_y then LDX $1000,Y (0x1000+0x0005=0x1005)
        assert_eq!(cpu.reg_x, 0x07);
    }

    // ---------- LDY tests

    #[test]
    fn test_0xa0_ldy_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0x00]);
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa4_ldy_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x09;
        cpu.load_and_run(vec![0xa4, 0x05, 0x00]);
        assert_eq!(cpu.reg_y, 0x09);
    }

    #[test]
    fn test_0xb4_lda_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x09] = 0x07;
        cpu.load_and_run(vec![0xa2, 0x05, 0xb4, 0x04, 0x00]);
        assert_eq!(cpu.reg_y, 0x07);
    }

    #[test]
    fn test_0xac_ldy_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x07;
        cpu.load_and_run(vec![0xac, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_y, 0x07);
    }

    #[test]
    fn test_0xbc_ldy_absolute_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa2, 0x05, 0xbc, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_y, 0x07);
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

    // ---------- ADC tests
    #[test]
    fn test_0x69_adc_cause_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x69, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_cause_positive_into_negative_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0x00]); // 0111 1111 + 0000 0001 => 127 + 1 => -128
        assert_eq!(cpu.reg_a, 0x80); // 1000 0000 => 0x80
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_cause_negative_into_positive_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x69, 0x80, 0x00]); // 1111 1111 + 1000 0000 => -1 - 128 => 127
        assert_eq!(cpu.reg_a, 0x7f); // 0111 1111 => 0x7F
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x69, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x65_adc_immediate_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x09;
        cpu.load_and_run(vec![0x65, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x09);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x75_adc_immediate_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x09] = 0x07;
        cpu.load_and_run(vec![0xa2, 0x05, 0x75, 0x04, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6d_adc_immediate_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x07;
        cpu.load_and_run(vec![0x6d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x7d_adc_immediate_absolute_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa2, 0x05, 0x7d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x79_adc_immediate_absolute_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.load_and_run(vec![0xa0, 0x05, 0x79, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x61_adc_immediate_indirect_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x07;
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa2, 0x05, 0x61, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0x71_lda_indirect_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x100a] = 0x07;
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa0, 0x05, 0x71, 0x0a, 0x00]);
        assert_eq!(cpu.reg_a, 0x07);
    }

    // ---------- AND tests

    #[test]
    fn test_0x29_and_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x29, 0x0F, 0x00]); // LDA #$FF; AND #$0F => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x25_and_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x0F;
        cpu.load_and_run(vec![0xa9, 0xff, 0x25, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x35_and_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x09] = 0x0F;
        cpu.load_and_run(vec![0xa2, 0x04, 0xa9, 0xff, 0x35, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x2d_and_absolute() {
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x0F;
        cpu.load_and_run(vec![0xa9, 0xff, 0x2d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x3d_and_absolute_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x0F;
        cpu.load_and_run(vec![0xa2, 0x05, 0xa9, 0xff, 0x3d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x39_and_absolute_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x0F;
        cpu.load_and_run(vec![0xa0, 0x05, 0xa9, 0xff, 0x39, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x21_and_indirect_x() {
        let mut cpu = CPU::new();
        cpu.memory[0x1005] = 0x0F;
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa2, 0x05, 0xa9, 0xff, 0x21, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x31_and_indirect_y() {
        let mut cpu = CPU::new();
        cpu.memory[0x100a] = 0x0F;
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa0, 0x05, 0xa9, 0xff, 0x31, 0x0a, 0x00]);
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x29_and_test_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x29, 0x00, 0x00]);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x29_and_test_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x29, 0x80, 0x00]); // 1111_1111 & 1000_0000
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- ASL tests

    #[test]
    fn test_0x0a_asl_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x0a, 0x00]); // LDA #$01; ASL
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x06_asl_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x01;
        cpu.load_and_run(vec![0x06, 0x05, 0x00]);
        assert_eq!(cpu.memory[0x05], 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x81, 0x0a, 0x00]); // 1000_0000 => 0000_0010
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x0a, 0x00]); // 0000_0000 => 0000_0000
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x40, 0x0a, 0x00]); // 0100_0000 => 1000_0000
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // This test, I'm not sure if it's true or not, can't find conclusive information that the zero flag is only set if its the accumulator
    // #[test]
    // fn test_0x06_asl_zero_page_doesnt_change_zero_flag() {
    //     let mut cpu = CPU::new();
    //     cpu.memory[0x05] = 0x80;
    //     cpu.load_and_run(vec![0x06, 0x05, 0x00]);
    //     assert_eq!(cpu.memory[0x05], 0x00);
    //     assert!(!cpu.is_zero_flag_set());
    //     assert!(!cpu.is_negative_flag_set());
    //     assert!(cpu.is_carry_flag_set());
    // }

    // ---------- SEC tests

    #[test]
    fn test_0x38_sec() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0x00]);
        assert!(cpu.is_carry_flag_set());
    }

    // ---------- BCC tests

    #[test]
    fn test_0x90_bcc() {
        // Program counter starts at 0x8000, read two instructions therefore it would be 8002
        // Then add 10 to the PC
        // Then read the next instruction which would be 0x00 since the memory is empty
        // Therefore program counter= 0x8000 + 0x02 + 0x10 + 0x01
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x90, 0x10]);
        assert_eq!(cpu.program_counter, 0x8013);
    }

    // ---------- BCS tests
    #[test]
    fn test_0xb0_bcs() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xb0, 0x10]);
        assert_eq!(cpu.program_counter, 0x8014);
    }

    // ---------- BEQ tests
    #[test]
    fn test_0xf0_beq() {
        //LDA 0x00 to set the zero flag, better to use CMP in the future when implemented
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xf0, 0x10]);
        assert_eq!(cpu.program_counter, 0x8015);
    }

    // ---------- BNE tests
    #[test]
    fn test_0xd0_bne_zero_not_set() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0xd0, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8015);
    }

    // ---------- BNE tests
    #[test]
    fn test_0xd0_bne_zero_set() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xd0, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    // ---------- BMI tests
    #[test]
    fn test_0x30_bmi_is_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x30, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_0x30_bmi_is_positive() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x30, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    // ---------- BPL tests
    #[test]
    fn test_0x10_bpl_is_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x10, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_0x10_bpl_is_positive() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x10, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8015);
    }

    // ---------- BVC tests
    #[test]
    fn test_0x50_bvc_no_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x50, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_0x50_bvc_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0x50, 0x10, 0x00]); //Cause overflow then try branching
        assert_eq!(cpu.program_counter, 0x8007);
        assert!(cpu.is_overflow_flag_set());
    }

    // ---------- BVS tests
    #[test]
    fn test_0x70_bvs_no_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x70, 0x10, 0x00]);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_0x70_bvs_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0x70, 0x10, 0x00]); //Cause overflow then try branching
        assert_eq!(cpu.program_counter, 0x8017);
        assert!(cpu.is_overflow_flag_set());
    }

    // ---------- BIT tests
    #[test]
    fn test_0x24_bit_result_zero() {
        //LDA 0x00 to set the zero flag, better to use CMP in the future when implemented
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x00;
        cpu.load_and_run(vec![0xa9, 0xFF, 0x24, 0x05]);
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
    }

    #[test]
    fn test_0x2c_bit_overflow_flag_set() {
        //LDA 0x00 to set the zero flag, better to use CMP in the future when implemented
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x40; // 0100_0000
        cpu.load_and_run(vec![0xa9, 0xFF, 0x2c, 0x00, 0x10]);
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
    }

    #[test]
    fn test_0x2c_bit_negative_flag_set() {
        //LDA 0x00 to set the zero flag, better to use CMP in the future when implemented
        let mut cpu = CPU::new();
        cpu.memory[0x1000] = 0x80; // 1000_0000
        cpu.load_and_run(vec![0xa9, 0xFF, 0x2c, 0x00, 0x10]);
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
    }

    // ---------- CLC tests
    #[test]
    fn test_0x18_clc_clear_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x69, 0x05, 0x18, 0x00]);
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- CLV tests
    #[test]
    fn test_0xb8_clv_clear_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0xb8, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- CLD tests
    #[test]
    fn test_0xd8_cld_clear_decimal() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xf8, 0xd8, 0x00]);
        assert!(!cpu.is_decimal_flag_set())
    }

    // ---------- CLI tests
    #[test]
    fn test_0x58_cli_clear_decimal() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x78, 0x58, 0x00]);
        assert!(!cpu.is_interrupt_flag_set())
    }

    // ---------- SED tests
    #[test]
    fn test_0xf8_sed_set_decimal() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xf8, 0x00]);
        assert!(cpu.is_decimal_flag_set())
    }

    // ---------- SEI tests
    #[test]
    fn test_0x78_sei_set_interupt() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x78, 0x00]);
        assert!(cpu.is_interrupt_flag_set())
    }

    // ---------- CMP tests
    #[test]
    fn test_0xc9_cmp_a_greater_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xc9, 0x01]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc9_cmp_a_equal_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xc9, 0x05]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc9_cmp_a_less_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xc9, 0x06]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- CPX tests
    #[test]
    fn test_0xe0_cpx_x_greater_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0xe0, 0x01]);
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe0_cpx_x_equal_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0xe0, 0x05]);
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe0_cpx_x_less_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0xe0, 0x06]);
        assert_eq!(cpu.reg_x, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- CPY tests
    #[test]
    fn test_0xc0_cpy_y_greater_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0xc0, 0x01]);
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc0_cpy_y_equal_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0xc0, 0x05]);
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc0_cpy_y_less_than_m() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0xc0, 0x06]);
        assert_eq!(cpu.reg_y, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- LDA tests
    #[test]
    fn test_0x85_sta_zero_page_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x85, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x95_sta_zero_page_x_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xa2, 0x02, 0x95, 0x0d, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x8d_sta_absolute_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x8d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.memory[0x1000], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x9d_sta_absolute_x_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xa2, 0x02, 0x9d, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.memory[0x1002], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x99_sta_absolute_y_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xa0, 0x02, 0x99, 0x00, 0x10, 0x00]);
        assert_eq!(cpu.memory[0x1002], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x81_sta_indirect_x_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa9, 0x05, 0xa2, 0x05, 0x81, 0x05, 0x00]);
        assert_eq!(cpu.memory[0x1005], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x91_sta_indirect_y_store_accumulator() {
        let mut cpu = CPU::new();
        cpu.memory[0x000a] = 0x05;
        cpu.memory[0x000b] = 0x10;
        cpu.load_and_run(vec![0xa9, 0x05, 0xa0, 0x05, 0x91, 0x0a, 0x00]);
        assert_eq!(cpu.memory[0x100a], 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    // ---------- STX tests
    #[test]
    fn test_0x86_stx_zero_page_store_reg_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0x86, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x05);
        assert_eq!(cpu.reg_x, 0x05);
    }

    // ---------- STY tests
    #[test]
    fn test_0x84_sty_zero_age_store_reg_y() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0x84, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x05);
        assert_eq!(cpu.reg_y, 0x05);
    }

    // ---------- DEC tests
    #[test]
    fn test_0xc6_dec_zero_page_normal_decrement() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x04);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc6_dec_zero_page_decrement_to_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0x00);
        assert_eq!(cpu.reg_a, 0x01);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc6_dec_zero_page_decrement_to_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        assert_eq!(cpu.memory[0x0f], 0xFF);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- DEX tests
    #[test]
    fn test_0xca_dex_normal_decrement() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0xca, 0x00]);
        assert_eq!(cpu.reg_x, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xca_dex_decrement_to_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x01, 0xca, 0x00]);
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xca_dex_decrement_to_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00, 0xca, 0x00]);
        assert_eq!(cpu.reg_x, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- EOR tests
    #[test]
    fn test_0x49_eor_normal_exclusive_or() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xaa, 0x49, 0xF5, 0x00]); // 1010_1010 exlusive or 1111_0101 = 0101_1111
        assert_eq!(cpu.reg_a, 0x5F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x49_eor_normal_exclusive_or_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x49, 0xFF, 0x00]); // 1111_1111 exlusive or 1111_1111 = 0
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x49_eor_normal_exclusive_or_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7F, 0x49, 0xFF, 0x00]); // 0111_1111 exlusive or 1111_1111 = 0
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- INC tests
    #[test]
    fn test_0xe6_inc_zero_page_normal_increment() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x85, 0x0f, 0xe6, 0x0f, 0x00]); // Store 0x05 in memory 0x0f then increment
        assert_eq!(cpu.memory[0x0f], 0x06);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe6_inc_zero_page_increment_to_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x85, 0x0f, 0xe6, 0x0f, 0x00]); // 0xFF +1 => 0x00
        assert_eq!(cpu.memory[0x0f], 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe6_inc_zero_page_increment_to_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7F, 0x85, 0x0f, 0xe6, 0x0f, 0x00]); // 01111111 +1 => 1000_0000
        assert_eq!(cpu.memory[0x0f], 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- INX tests
    #[test]
    fn test_0xe8_inx_normal_increment() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x20, 0xaa, 0xe8, 0x00]; // Transfer value 0x20 (decimal: 32) into accumulator and TAX and the increment

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_x, 33);
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

    // ---------- INY tests
    #[test]
    fn test_0xc8_iny_normal_increment() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa0, 0x20, 0xc8, 0x00];

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 33);
    }
    #[test]
    fn test_0xc8_iny_increment_into_negative() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa0, 0x7F, 0xc8, 0x00];

        cpu.load_and_run(program);
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 0x80);
    }
    #[test]
    fn test_0xc8_iny_increment_into_zero() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa0, 0xFF, 0xc8, 0x00];

        cpu.load_and_run(program);
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 0);
    }

    // ---------- JMP tests
    #[test]
    fn test_0x4c_jmp_normal_jump() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0x4c, 0x00, 0x10]; // JMP $1000
        cpu.load_and_run(program);

        assert_eq!(cpu.program_counter, 0x1001);
    }

    #[test]
    fn test_0x6c_jmp_normal_jump_indirect() {
        let mut cpu: CPU = CPU::new();
        cpu.memory[0x1000] = 0x00;
        cpu.memory[0x1001] = 0x20;
        let program: Vec<u8> = vec![0x6c, 0x00, 0x10];
        cpu.load_and_run(program);

        assert_eq!(cpu.program_counter, 0x2001);
    }

    #[test]
    fn test_0x6c_jmp_normal_jump_indirect_boundary_bug() {
        let mut cpu: CPU = CPU::new();
        cpu.memory[0x1050] = 0x00;
        cpu.memory[0x1051] = 0x20;

        cpu.memory[0x1000] = 0x10;
        cpu.memory[0x10FF] = 0x50;
        // Instead of loading addresses at $10FF and $1100 where 10FF is the lowbyte and 1100 is the high byte
        // It's loading the low byte from $10ff and the high byte from $1000 therefore reading the address stored at $1050
        let program: Vec<u8> = vec![0x6c, 0xFF, 0x10];
        cpu.load_and_run(program);

        assert_eq!(cpu.program_counter, 0x2001);
    }

    // ---------- JSR tests
    #[test]
    fn test_0x20_jsr_normal_jump() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0x20, 0x00, 0x10];
        cpu.load_and_run(program);

        // Program counter starts at 0x8000, we read 3 bytes so its 8003 when we finish the JSR command but we push the pc minus 1
        // Therefore we should see the high byte 0x80 in the stack followed by the low byte 0x02
        // The program counter should also be at 10001
        assert_eq!(cpu.memory[0x01FF], 0x80);
        assert_eq!(cpu.memory[0x01FE], 0x02);
        assert_eq!(cpu.program_counter, 0x1001);
    }

    // ---------- LSR tests
    #[test]
    fn test_0x4a_lsr_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x02, 0x4a, 0x00]); // LDA #$02; LSR
        assert_eq!(cpu.reg_a, 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x46_lsr_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x02;
        cpu.load_and_run(vec![0x46, 0x05, 0x00]);
        assert_eq!(cpu.memory[0x05], 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x4a_lsr_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x81, 0x4a, 0x00]); // 1000_0001 => 0100_0000
        assert_eq!(cpu.reg_a, 0x40);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x4a_lsr_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x4a, 0x00]); // 0000_0001 => 0000_0000
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    // ---------- NOP tests
    #[test]
    fn test_0xea_nop() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xEA, 0x00];
        cpu.load_and_run(program);
        
        assert_eq!(cpu.program_counter, 0x8002);
    }

    // ---------- ORA tests
    #[test]
    fn test_0x09_ora_normal_inclusive_or() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x2a, 0x09, 0x75, 0x00]); // 0010_1010 inclusive or 0111_0101 = 0111_1111
        assert_eq!(cpu.reg_a, 0x7F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x09_ora_normal_inclusive_or_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x09, 0x00, 0x00]); // 0 inclusive or 0 = 0
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x09_ora_normal_inclusive_or_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xAA, 0x09, 0x0F, 0x00]); // 1010_1010 inclusive or 0000_1111 = 1010_1111
        assert_eq!(cpu.reg_a, 0xAF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- PHA tests
    #[test]
    fn test_0x48_pha_push_accumulator() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x80, 0x48, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.memory[0x01FF], 0x80);
    }

    // ---------- PHP tests
    #[test]
    fn test_0x09_php_push_status_flag() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0x38, 0xf8, 0x78, 0x08, 0x00]; //Setting carry, interrupt and decimal flags
        cpu.load_and_run(program);

        // Stack should contain: 0011_1101
        assert_eq!(cpu.memory[0x01FF], 0x3D);
    }

    // ---------- PLA tests
    #[test]
    fn test_0x68_pla_pull_stack_into_accumulator() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0xa9, 0x80, 0x48, 0xa9, 0x01, 0x68, 0x00]; // Push 0x80 into stack, then replace accumulator with 01 then pull stack to accumulator to make sure its back to 0x80
        cpu.load_and_run(program);

        assert_eq!(cpu.reg_a, 0x80);
        assert_eq!(cpu.stack_ptr, 0xFF);
    }

    // ---------- PLP tests
    #[test]
    fn test_0x28_plp_pull_status_flag() {
        let mut cpu: CPU = CPU::new();
        let program: Vec<u8> = vec![0x38, 0xf8, 0x78, 0x08, 0x18, 0xd8, 0x58, 0x28, 0x00]; //Setting carry, interrupt and decimal flags, then clearing those flags and pulling the status
        cpu.load_and_run(program);

        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_decimal_flag_set());
        assert!(cpu.is_interrupt_flag_set());
    }

    // ---------- ROL tests
    #[test]
    fn test_0x2a_rol_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x02, 0x2a, 0x00]); // LDA #$02; ROL
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x26_rol_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x02;
        cpu.load_and_run(vec![0x26, 0x05, 0x00]);
        assert_eq!(cpu.memory[0x05], 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_bit_7_to_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x81, 0x2a, 0x00]); // 1000_0001 => 0000_0010
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_carry_flag_to_bit_0() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x38, 0x2a, 0x00]); // set carry flag then  0000_0001 => 0000_0011
        assert_eq!(cpu.reg_a, 0x03);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x2a, 0x00]); // set carry flag then  1000_0000 => 0000_0000
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x40, 0x2a, 0x00]); // set carry flag then  0100_0000 => 1000_0000
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- ROR tests
    #[test]
    fn test_0x6a_ror_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x02, 0x6a, 0x00]); // LDA #$02; ROR
        assert_eq!(cpu.reg_a, 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x66_ror_zero_page() {
        let mut cpu = CPU::new();
        cpu.memory[0x05] = 0x02;
        cpu.load_and_run(vec![0x66, 0x05, 0x00]);
        assert_eq!(cpu.memory[0x05], 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_bit_0_to_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x81, 0x6a, 0x00]); // 1000_0001 => 0100_0000
        assert_eq!(cpu.reg_a, 0x40);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_carry_flag_to_bit_7_and_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x02, 0x38, 0x6a, 0x00]); // set carry flag then  0000_0010 => 1000_0001
        assert_eq!(cpu.reg_a, 0x81);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0x6a, 0x00]); // set carry flag then  0000_0001 => 0000_0000
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

}