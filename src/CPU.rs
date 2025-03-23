use std::ops::Add;
use hex;

use crate::AddressingModes::AddressingMode;
use crate::OpCodes::{Mnemonic, OpCode, OPCODES_MAP};
use crate::OpCodes::Mnemonic::*;
use crate::bus::Bus;
use crate::memory::Memory;

pub struct CPU {
    // TODO: need to set these private and make getters/setters
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub state: u8, // bit flags: NV-BDIZC where N=negative V=overflow, B=break, D=decimal mode, I=interrupt, Z=zero, C=carry
    pub program_counter: u16,
    pub stack_ptr: u8,
    pub bus: Bus,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            state: 0b0010_0100,
            program_counter: 0x8000,
            stack_ptr: 0xFD, // In 6502 the stack pointer always starts at 0x01ff and decrements
            bus: bus,
        }
    }
    
    //-- Memory read and write methods

    pub fn mem_read(&self, address: u16) -> u8 {
        self.bus.mem_read(address)
    }

    pub fn mem_write(&mut self, address: u16, value:u8) {
        self.bus.mem_write(address, value);
    }

    pub fn mem_read_u16(&self, address: u16) -> u16 {
        self.bus.mem_read_u16(address)
    }

    pub fn mem_write_u16(&mut self, address: u16, value: u16) {
        self.bus.mem_write_u16(address, value);
    }

    //-- Core CPU methods

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.state = 0b0010_0100; // 5th bit always set
        self.stack_ptr = 0xFD; // In 6502 the stack pointer always starts at 0x01ff

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // Load the program starting at 0x8000 since that's where program ROM is allocated according to NES specs
        for i in 0..program.len() {
            self.mem_write(0x8000 + i as u16, program[i]);
        }
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load_and_run_snake_game<F>(&mut self, program: Vec<u8>, mut callback: F)
    where F: FnMut(&mut CPU),
    {
        // Seperate function for the snake game since it expects the program code to be in a different location
        for i in 0..program.len() {
            self.mem_write(0x0600 + i as u16, program[i]);
        }
        self.mem_write_u16(0xFFFC, 0x0600);
        self.reset();
        self.run_with_callback(callback);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_|{});
    }

    pub fn trace(&self) -> String {
        let curr_instruction: u8 = self.mem_read(self.program_counter);
        let opcode: &OpCode = OPCODES_MAP.get(&curr_instruction).expect(&format!("Instruction: {curr_instruction} not found"));
        let opcodes: Vec<u8> = {
            let num_bytes: u16 = opcode.get_num_bytes() as u16;
            let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes as usize);
            bytes.push(self.mem_read(self.program_counter));

            for i in 1..num_bytes {
                bytes.push(self.mem_read(self.program_counter+i));
            }

            bytes
        };

        let mneumonic_str: String = opcode.get_instruction().to_string();
        let (memory_address, value_stored_at_address) = match opcode.get_mode() {
            AddressingMode::Immediate | AddressingMode::NoneAddressing | AddressingMode::Accumulator => (0,0),
            _ => {
                let addr: u16 = opcode.get_mode().get_operand_address_from_program_counter(self.program_counter+1, self);
                let value: u8 = self.mem_read(addr);
                (addr, value)
            }
        };

        let incremented_program_counter: u16 = self.program_counter +1;
        let mode: &AddressingMode = opcode.get_mode();
        let parameter_in_original_assembly_code: String = match mode {
            AddressingMode::Immediate => {
                format!("#${:02X}",self.mem_read(incremented_program_counter))
            },
            AddressingMode::ZeroPage => {
                let zero_page: u8 = self.mem_read(incremented_program_counter);
                format!("${:02X} = {:02X}",
                zero_page, value_stored_at_address)
            },
            AddressingMode::ZeroPage_X => {
                let zero_page: u8 = self.mem_read(incremented_program_counter);
                format!("${:02X},X @ {:02X} = {:02X}",
                zero_page, memory_address, value_stored_at_address)
            },
            AddressingMode::ZeroPage_Y => {
                let zero_page: u8 = self.mem_read(incremented_program_counter);
                format!("${:02X},Y @ {:02X} = {:02X}",
                zero_page, memory_address, value_stored_at_address)
            },
            AddressingMode::Absolute => {
                let absolute: u16 = self.mem_read_u16(incremented_program_counter);
                match opcode.get_instruction() {
                    Mnemonic::JMP | Mnemonic::JSR => {
                        format!("${:04X}",
                        absolute)
                    }
                    _ => {
                        format!("${:04X} = {:02X}",
                    absolute, value_stored_at_address)
                    }
                }
            },
            AddressingMode::Indirect => {
                let absolute: u16 = self.mem_read_u16(incremented_program_counter);
                format!("$({:04X}) = {:04X}",
                absolute, value_stored_at_address)
            },
            AddressingMode::Absolute_X => {
                let absolute: u16 = self.mem_read_u16(incremented_program_counter);
                format!("${:04X},X @ {:04X} = {:02X}",
                absolute, memory_address, value_stored_at_address)
            },
            AddressingMode::Absolute_Y => {
                let absolute: u16 = self.mem_read_u16(incremented_program_counter);
                format!("${:04X},Y @ {:04X} = {:02X}",
                absolute, memory_address, value_stored_at_address)
            },
            AddressingMode::Indirect_X => {
                let indirect_addr: u8 = self.mem_read(incremented_program_counter);
                format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                indirect_addr, indirect_addr.wrapping_add(self.reg_x), memory_address, value_stored_at_address)
            },
            AddressingMode::Indirect_Y => {
                let indirect_addr: u8 = self.mem_read(incremented_program_counter);
                let referenced_addr: u16 = self.mem_read_u16(indirect_addr as u16);
                format!("(${:02X}),Y = {:04x} @ {:04x} = {:02X}",
                indirect_addr, referenced_addr, memory_address, value_stored_at_address)
            },
            AddressingMode::NoneAddressing => {
                String::new()
            },
            AddressingMode::Relative => {
                let jump_to_u16: u16 = self.program_counter.wrapping_add(2).wrapping_add(value_stored_at_address as u16);
                format!("${:04X}", jump_to_u16)
            },
            AddressingMode::Accumulator => {
                'A'.to_string()
            },
        };

        let opcodes_string: String = opcodes
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .join(" ");

        let instruction_string: String = format!("{:04X}  {:08}  {:03} {}",
        self.program_counter, opcodes_string, mneumonic_str, parameter_in_original_assembly_code);

        format!("{:47} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        instruction_string, self.reg_a, self.reg_x, self.reg_y, self.state, self.stack_ptr)


    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) 
    where F: FnMut(&mut CPU),
    {
        
        loop {
            callback(self);
            let curr_instruction: u8 = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let initial_program_counter: u16 = self.program_counter;

            let opcode: &OpCode = OPCODES_MAP.get(&curr_instruction).expect(&format!("Instruction: {curr_instruction} not found"));
            let mode: &AddressingMode = opcode.get_mode();
            let instruction: Mnemonic = opcode.get_instruction();

            match instruction {
                ADC => self.adc(mode),
                SBC => self.sbc(mode),
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
                RTI => self.rti(),
                RTS => self.rts(),
                PHP => self.php(),
                PLP => self.plp(),
                PLA => self.pla(),
                PHA => self.pha(),
                DEX => self.dex(),
                DEY => self.dey(),
                TAX => self.tax(),
                TAY => self.tay(),
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
                TSX => self.tsx(),
                TXA => self.txa(),
                TXS => self.txs(),
                TYA => self.tya(),

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

    fn tay(&mut self) {
        self.reg_y = self.reg_a;
        self.set_zero_and_negative_flag(self.reg_y);
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

    fn sbc(&mut self, mode: &AddressingMode) {
        let param: u8 = self.mem_read(mode.get_operand_address(&self));
        let negative_param: u8 = (!param).wrapping_add(1); // 2's complement

        let mut sum_as_u16: u16 = self.reg_a as u16 + negative_param as u16;
        if !self.is_carry_flag_set() {
            sum_as_u16 -= 1;
        }

        let sum_as_u8: u8 = sum_as_u16 as u8; //gets lower 8 bits during conversion

        let carry_flag: bool = self.reg_a >= {param + if self.is_carry_flag_set() {0} else {1}};
        self.set_carry_flag(carry_flag);
        self.set_overflow_flag(
            ((self.reg_a ^ param) & (self.reg_a ^ sum_as_u8) & 0x80) != 0
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
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let new_val: u8 = param << 1;

        self.set_carry_flag((param & 0b1000_0000) != 0);

        self.set_negative_flag((new_val & 0b1000_0000) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing | AddressingMode::Accumulator = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
        
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let new_val: u8 = param >> 1;

        self.set_carry_flag((param & 0b0000_0001) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing | AddressingMode::Accumulator = mode {
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
        // The reason it's minus one is that in a real 6502 cpu, the program counter would have been pointing to the last byte in the 3 byte instructions of jsr
        // It therefore would have pushed the return point for RTS -1 so I'm just keeping it consistent with the actual cpu
        self.push_to_stack_u16(self.program_counter +2 -1); 
        self.program_counter = param;
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let mut new_val: u8 = param << 1;

        if self.is_carry_flag_set() {
            new_val = new_val | 0b_0000_0001;
        }

        self.set_carry_flag((param & 0b1000_0000) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing | AddressingMode::Accumulator = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let param: u8 = match mode {
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => self.reg_a,
            _ => self.mem_read(mode.get_operand_address(&self))
        };

        let mut new_val: u8 = param >> 1;

        if self.is_carry_flag_set() {
            new_val = new_val | 0b_1000_0000;
        }

        self.set_carry_flag((param & 0b0000_0001) != 0);
        self.set_zero_and_negative_flag(new_val);

        if let AddressingMode::NoneAddressing | AddressingMode::Accumulator = mode {
            self.reg_a = new_val;
        } else {
            self.mem_write(mode.get_operand_address(&self), new_val);
        }
    }

    fn tsx(&mut self) {
        self.reg_x = self.stack_ptr;
        self.set_zero_and_negative_flag(self.reg_x);
    }

    fn txs(&mut self) {
        self.stack_ptr = self.reg_x;
    }

    fn txa(&mut self) {
        self.reg_a = self.reg_x;
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn tya(&mut self) {
        self.reg_a = self.reg_y;
        self.set_zero_and_negative_flag(self.reg_a);
    }

    fn rti(&mut self) {
        self.plp();
        self.program_counter = self.pull_from_stack_u16();
    }

    fn rts(&mut self) {
        self.program_counter = self.pull_from_stack_u16() +1; // Need to compensate for the -1 subtracted during the JSR command
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
        self.set_break_flag(false);
        self.set_bit_5_flag(true);
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
        // Stack lives between 0x100 and 0x1ff, stack_ptr starts at 0xff and decrements with each push to the stack
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

    fn pull_from_stack_u16(&mut self) -> u16 {
        let value_low: u16 = self.pull_from_stack_u8() as u16;
        let value_high: u16 = (self.pull_from_stack_u8() as u16) << 8;
        value_high | value_low
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
        let param: i8 = self.mem_read(mode.get_operand_address(&self)) as i8;
        self.program_counter =  self.program_counter.wrapping_add(1).wrapping_add(param as u16); // Reading the byte containing the param and then add the jump
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

    fn set_bit_5_flag(&mut self, break_flag: bool) {
        if break_flag {
            self.state |= 0b0010_0000;
        } else {
            self.state &= 0b1101_1111;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::rom::{MirroringType, Rom};
    use crate::bus::Bus;

    fn convert_program_to_cpu(mut program: Vec<u8>) -> CPU {
        program.resize(0x4000, 0); // Resizing to 16kb to prevent index out of range for some tests like branching tests
        let rom: Rom = Rom { program_rom: program, character_rom: vec![], mapper: 0, mirroring: MirroringType::Vertical};
        let bus: Bus = Bus::new(rom);

        let cpu: CPU = CPU::new(bus);
        cpu
    }
   
    // ---------- LDA tests
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa5_lda_zero_page() {
        let mut cpu = convert_program_to_cpu(vec![0xa5, 0x05, 0x00]); // Load value at memory location 0x05 using lda
        cpu.mem_write(0x05, 0x09); // Assign value 9 to memory location 0x05
        cpu.run();
        assert_eq!(cpu.reg_a, 0x09);
    }

    #[test]
    fn test_0xb5_lda_zero_page_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xb5, 0x04, 0x00]);  // Load 0x05 into reg_x, then load A with the value stored at 0x04 + reg_x (0x04+0x05=0x09) which has value 7
        cpu.mem_write(0x09, 0x07); // Assign value 7 to memory location 0x09 (0x05 + 0x04)
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute() {
        let mut cpu = convert_program_to_cpu(vec![0xad, 0x00, 0x10, 0x00]); // LDA $1000 (little endian so its bytes 0x00 and then 0x10)
        cpu.mem_write(0x1000, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xbd, 0x00, 0x10, 0x00]); //Load 0x05 into reg_x then LDA $1000,X (0x1000+0x0005=0x1005)
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_absolute_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xb9, 0x00, 0x10, 0x00]); //Load 0x05 into reg_y then LDA $1000,Y (0x1000+0x0005=0x1005)
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_indirect_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xa1, 0x05, 0x00]); //Load 0x05 into reg_x then LDA ($05,X) (0x05 + 0x05 = 0x0a => address referenced at 0x0a = 0x1005)
        cpu.mem_write(0x1005, 0x07);
        // Little endian storage of address 1005 (least significant [0x05] first then most [0x10])
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xad_lda_indirect_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x07, 0xb1, 0x0a, 0x00]); //Load 0x07 into reg_y then LDA ($0a),Y (addr referenced at 0x0a => 0x1005, add x (0x05) => 0x100c)
        cpu.mem_write(0x100c, 0x07);
        // Little endian storage of address 1005 (least significant [0x05] first then most [0x10])
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x00]);
        cpu.run();
        assert!(cpu.state & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0x00]); // Transfer value 0x80 into accumulator which corresponds to 0b1000_0000
        cpu.run();
        assert!(cpu.is_negative_flag_set());
    }
    
    // ---------- LDX tests

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa6_ldx_zero_page() {
        let mut cpu = convert_program_to_cpu(vec![0xa6, 0x05, 0x00]); // Load value at memory location 0x05 using lda
        cpu.mem_write(0x05, 0x09); // Assign value 9 to memory location 0x05
        cpu.run();
        assert_eq!(cpu.reg_x, 0x09);
    }

    #[test]
    fn test_0xb6_ldx_zero_page_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xb6, 0x04, 0x00]); // Load 0x05 into reg_y, then load A with the value stored at 0x04 + reg_y (0x04+0x05=0x09) which has value 7
        cpu.mem_write(0x09, 0x07); // Assign value 7 to memory location 0x09 (0x05 + 0x04)
        cpu.run();
        assert_eq!(cpu.reg_x, 0x07);
    }

    #[test]
    fn test_0xae_ldx_absolute() {
        let mut cpu = convert_program_to_cpu(vec![0xae, 0x00, 0x10, 0x00]); // LDX $1000 (little endian so its bytes 0x00 and then 0x10)
        cpu.mem_write(0x1000, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x07);
    }

    #[test]
    fn test_0xbe_ldx_absolute_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xbe, 0x00, 0x10, 0x00]); //Load 0x05 into reg_y then LDX $1000,Y (0x1000+0x0005=0x1005)
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x07);
    }

    // ---------- LDY tests

    #[test]
    fn test_0xa0_ldy_immediate_load_data() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.state & 0b0000_0010 == 0b00);
        assert!(cpu.state & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa4_ldy_zero_page() {
        let mut cpu = convert_program_to_cpu(vec![0xa4, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x09);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x09);
    }

    #[test]
    fn test_0xb4_lda_zero_page_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xb4, 0x04, 0x00]);
        cpu.mem_write(0x09, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x07);
    }

    #[test]
    fn test_0xac_ldy_absolute() {
        let mut cpu = convert_program_to_cpu(vec![0xac, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1000, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x07);
    }

    #[test]
    fn test_0xbc_ldy_absolute_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xbc, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x07);
    }

    // ---------- TAX tests
    #[test]
    fn test_0xaa_tax_a_is_zero() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0xaa, 0x00]); // Transfer value 0 into accumulator and TAX

        cpu.run();
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xaa_tax_a_is_negative() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0xaa, 0x00]); // Transfer value 0x80 into accumulator which corresponds to 0b1000_0000 and TAX

        cpu.run();
        assert_eq!(cpu.reg_x, 0x80);
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    #[test]
    fn test_0xaa_tax_neither_negative_or_zero() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x20, 0xaa, 0x00]); // Transfer value 0x20 (decimal: 32) into accumulator and TAX

        cpu.run();
        assert_eq!(cpu.reg_x, 0x20);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    // ---------- ADC tests
    #[test]
    fn test_0x69_adc_cause_carry() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x69, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_cause_positive_into_negative_overflow() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7f, 0x69, 0x01, 0x00]); // 0111 1111 + 0000 0001 => 127 + 1 => -128
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80); // 1000 0000 => 0x80
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_cause_negative_into_positive_overflow() {
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x69, 0x80, 0x00]); // 1111 1111 + 1000 0000 => -1 - 128 => 127
        cpu.run();
        assert_eq!(cpu.reg_a, 0x7f); // 0111 1111 => 0x7F
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x69_adc_immediate() {
        let mut cpu = convert_program_to_cpu(vec![0x69, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x65_adc_immediate_zero_page() {
        let mut cpu = convert_program_to_cpu(vec![0x65, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x09);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x09);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x75_adc_immediate_zero_page_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x75, 0x04, 0x00]);
        cpu.mem_write(0x09, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6d_adc_immediate_absolute() {
        let mut cpu = convert_program_to_cpu(vec![0x6d, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1000, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x7d_adc_immediate_absolute_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x7d, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x79_adc_immediate_absolute_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0x79, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1005, 0x07);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x61_adc_immediate_indirect_x() {
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x61, 0x05, 0x00]);
        cpu.mem_write(0x1005, 0x07);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    #[test]
    fn test_0x71_lda_indirect_y() {
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x07, 0x71, 0x0a, 0x00]);
        cpu.mem_write(0x100c, 0x07);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x07);
    }

    // ---------- AND tests

    #[test]
    fn test_0x29_and_immediate() {
        // LDA #$FF; AND #$0F => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x29, 0x0F, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x25_and_zero_page() {
        // LDA #$FF; AND $05 => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x25, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x0F);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x35_and_zero_page_x() {
        // LDX #$04; LDA #$FF; AND $05,X => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x04, 0xa9, 0xff, 0x35, 0x05, 0x00]);
        cpu.mem_write(0x09, 0x0F);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x2d_and_absolute() {
        // LDA #$FF; AND $1000 => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x2d, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1000, 0x0F);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x3d_and_absolute_x() {
        // LDX #$05; LDA #$FF; AND $1000,X => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xa9, 0xff, 0x3d, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1005, 0x0F);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x39_and_absolute_y() {
        // LDY #$05; LDA #$FF; AND $1000,Y => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xa9, 0xff, 0x39, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1005, 0x0F);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x21_and_indirect_x() {
        // LDX #$05; LDA #$FF; AND ($05,X) => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xa9, 0xff, 0x21, 0x05, 0x00]);
        cpu.mem_write(0x1005, 0x0F);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x31_and_indirect_y() {
        // LDY #$07; LDA #$FF; AND ($0A),Y => 1111_1111 & 0000_1111 => 0000_1111 => 0x0F
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x07, 0xa9, 0xff, 0x31, 0x0a, 0x00]);
        cpu.mem_write(0x100c, 0x0F);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x0F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x29_and_test_zero() {
        // LDA #$FF; AND #$00 => 1111_1111 & 0000_0000 => 0000_0000 => 0x00
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x29, 0x00, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x29_and_test_negative() {
        // LDA #$FF; AND #$80 => 1111_1111 & 1000_0000 => 1000_0000 => 0x80
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x29, 0x80, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- ASL tests

    #[test]
    fn test_0x0a_asl_accumulator() {
        // LDA #$01; ASL => 0000_0001 => 0000_0010 => 0x02
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0x0a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x06_asl_zero_page() {
        // ASL $05 => 0000_0001 => 0000_0010 => 0x02
        let mut cpu = convert_program_to_cpu(vec![0x06, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x01);
        cpu.run();
        assert_eq!(cpu.mem_read(0x05), 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_carry_flag() {
        // LDA #$81; ASL => 1000_0001 => 0000_0010 => 0x02
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x81, 0x0a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_zero_flag() {
        // LDA #$00; ASL => 0000_0000 => 0000_0000 => 0x00
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x0a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x0a_asl_negative_flag() {
        // LDA #$40; ASL => 0100_0000 => 1000_0000 => 0x80
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x40, 0x0a, 0x00]);
        cpu.run();
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
        // SEC => Set Carry Flag
        let mut cpu = convert_program_to_cpu(vec![0x38, 0x00]);
        cpu.run();
        assert!(cpu.is_carry_flag_set());
    }

    // ---------- BCC tests

    #[test]
    fn test_0x90_bcc() {
        // Program counter starts at 0x8000, read two instructions therefore it would be 8002
        // Then add 10 to the PC
        // Then read the next instruction which would be 0x00 since the memory is empty
        // Therefore program counter= 0x8000 + 0x02 + 0x10 + 0x01

        // BCC $10 => Branch if Carry Clear
        let mut cpu = convert_program_to_cpu(vec![0x90, 0x10]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8013);
    }

    // ---------- BCS tests
    #[test]
    fn test_0xb0_bcs() {
        // SEC; BCS $10 => Branch if Carry Set
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xb0, 0x10]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8014);
    }

    // ---------- BEQ tests
    #[test]
    fn test_0xf0_beq() {
        // LDA #$00; BEQ $10 => Branch if Equal (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0xf0, 0x10]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    // ---------- BNE tests
    #[test]
    fn test_0xd0_bne_zero_not_set() {
        // LDA #$01; BNE $10 => Branch if Not Equal (Zero Flag Not Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0xd0, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_0xd0_bne_zero_set() {
        // LDA #$00; BNE $10 => Branch if Not Equal (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0xd0, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8005);
    }

    // ---------- BMI tests
    #[test]
    fn test_0x30_bmi_is_negative() {
        // LDA #$FF; BMI $10 => Branch if Minus (Negative Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x30, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_0x30_bmi_is_positive() {
        // LDA #$00; BMI $10 => Branch if Minus (Negative Flag Not Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x30, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8005);
    }
    

    // ---------- BPL tests
    #[test]
    fn test_0x10_bpl_is_negative() {
        // LDA #$FF; BPL $10 => Branch if Plus (Negative Flag Not Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x10, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_0x10_bpl_is_positive() {
        // LDA #$00; BPL $10 => Branch if Plus (Negative Flag Not Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x10, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    // ---------- BVC tests
    #[test]
    fn test_0x50_bvc_no_overflow() {
        // LDA #$FF; BVC $10 => Branch if Overflow Clear
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x50, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_0x50_bvc_overflow() {
        // LDA #$7F; ADC #$01; BVC $10 => Branch if Overflow Clear (Overflow Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7f, 0x69, 0x01, 0x50, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8007);
        assert!(cpu.is_overflow_flag_set());
    }

    // ---------- BVS tests
    #[test]
    fn test_0x70_bvs_no_overflow() {
        // LDA #$FF; BVS $10 => Branch if Overflow Set (Overflow Flag Not Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x70, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_0x70_bvs_overflow() {
        // LDA #$7F; ADC #$01; BVS $10 => Branch if Overflow Set (Overflow Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7f, 0x69, 0x01, 0x70, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8017);
        assert!(cpu.is_overflow_flag_set());
    }

    // ---------- BIT tests
    #[test]
    fn test_0x24_bit_result_zero() {
        // LDA #$FF; BIT $05 => Test Bits in Memory with Accumulator (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x24, 0x05]);
        cpu.mem_write(0x05, 0x00);
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
    }

    #[test]
    fn test_0x2c_bit_overflow_flag_set() {
        // LDA #$FF; BIT $1000 => Test Bits in Memory with Accumulator (Overflow Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x2c, 0x00, 0x10]);
        cpu.mem_write(0x1000, 0x40); // 0100_0000
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
    }

    #[test]
    fn test_0x2c_bit_negative_flag_set() {
        // LDA #$FF; BIT $1000 => Test Bits in Memory with Accumulator (Negative Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x2c, 0x00, 0x10]);
        cpu.mem_write(0x1000, 0x80); // 1000_0000
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
    }

    // ---------- CLC tests
    #[test]
    fn test_0x18_clc_clear_carry() {
        // LDA #$FF; ADC #$05; CLC => Clear Carry Flag
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xff, 0x69, 0x05, 0x18, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- CLV tests
    #[test]
    fn test_0xb8_clv_clear_overflow() {
        // LDA #$7F; ADC #$01; CLV => Clear Overflow Flag
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7f, 0x69, 0x01, 0xb8, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- CLD tests
    #[test]
    fn test_0xd8_cld_clear_decimal() {
        // SED; CLD => Clear Decimal Flag
        let mut cpu = convert_program_to_cpu(vec![0xf8, 0xd8, 0x00]);
        cpu.run();
        assert!(!cpu.is_decimal_flag_set())
    }

    // ---------- CLI tests
    #[test]
    fn test_0x58_cli_clear_decimal() {
        // SEI; CLI => Clear Interrupt Flag
        let mut cpu = convert_program_to_cpu(vec![0x78, 0x58, 0x00]);
        cpu.run();
        assert!(!cpu.is_interrupt_flag_set())
    }

    // ---------- SED tests
    #[test]
    fn test_0xf8_sed_set_decimal() {
        // SED => Set Decimal Flag
        let mut cpu = convert_program_to_cpu(vec![0xf8, 0x00]);
        cpu.run();
        assert!(cpu.is_decimal_flag_set())
    }

    // ---------- SEI tests
    #[test]
    fn test_0x78_sei_set_interupt() {
        // SEI => Set Interrupt Flag
        let mut cpu = convert_program_to_cpu(vec![0x78, 0x00]);
        cpu.run();
        assert!(cpu.is_interrupt_flag_set())
    }

    // ---------- CMP tests
    #[test]
    fn test_0xc9_cmp_a_greater_than_m() {
        // LDA #$05; CMP #$01 => Compare Accumulator with Memory (A > M)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xc9, 0x01]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc9_cmp_a_equal_m() {
        // LDA #$05; CMP #$05 => Compare Accumulator with Memory (A == M)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xc9, 0x05]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc9_cmp_a_less_than_m() {
        // LDA #$05; CMP #$06 => Compare Accumulator with Memory (A < M)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xc9, 0x06]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- CPX tests
    #[test]
    fn test_0xe0_cpx_x_greater_than_m() {
        // LDX #$05; CPX #$01 => Compare X Register with Memory (X > M)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xe0, 0x01]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe0_cpx_x_equal_m() {
        // LDX #$05; CPX #$05 => Compare X Register with Memory (X == M)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xe0, 0x05]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe0_cpx_x_less_than_m() {
        // LDX #$05; CPX #$06 => Compare X Register with Memory (X < M)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xe0, 0x06]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- CPY tests
    #[test]
    fn test_0xc0_cpy_y_greater_than_m() {
        // LDY #$05; CPY #$01 => Compare Y Register with Memory (Y > M)
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xc0, 0x01]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc0_cpy_y_equal_m() {
        // LDY #$05; CPY #$05 => Compare Y Register with Memory (Y == M)
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xc0, 0x05]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x05);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc0_cpy_y_less_than_m() {
        // LDY #$05; CPY #$06 => Compare Y Register with Memory (Y < M)
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0xc0, 0x06]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x05);
        assert!(!cpu.is_carry_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- STA tests
    #[test]
    fn test_0x85_sta_zero_page_store_accumulator() {
        // LDA #$05; STA $0F => Store Accumulator in Memory
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0x85, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x95_sta_zero_page_x_store_accumulator() {
        // LDA #$05; LDX #$02; STA $0D,X => Store Accumulator in Memory (Zero Page,X)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xa2, 0x02, 0x95, 0x0d, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x8d_sta_absolute_store_accumulator() {
        // LDA #$05; STA $1000 => Store Accumulator in Memory (Absolute)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0x8d, 0x00, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x1000), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x9d_sta_absolute_x_store_accumulator() {
        // LDA #$05; LDX #$02; STA $1000,X => Store Accumulator in Memory (Absolute,X)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xa2, 0x02, 0x9d, 0x00, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x1002), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x99_sta_absolute_y_store_accumulator() {
        // LDA #$05; LDY #$02; STA $1000,Y => Store Accumulator in Memory (Absolute,Y)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xa0, 0x02, 0x99, 0x00, 0x10, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x1002), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x81_sta_indirect_x_store_accumulator() {
        // LDA #$05; LDX #$05; STA ($05,X) => Store Accumulator in Memory (Indirect,X)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xa2, 0x05, 0x81, 0x05, 0x00]);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.mem_read(0x1005), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    #[test]
    fn test_0x91_sta_indirect_y_store_accumulator() {
        // LDA #$05; LDY #$05; STA ($0A),Y => Store Accumulator in Memory (Indirect),Y
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0xa0, 0x05, 0x91, 0x0a, 0x00]);
        cpu.mem_write(0x000a, 0x05);
        cpu.mem_write(0x000b, 0x10);
        cpu.run();
        assert_eq!(cpu.mem_read(0x100a), 0x05);
        assert_eq!(cpu.reg_a, 0x05);
    }

    // ---------- STX tests
    #[test]
    fn test_0x86_stx_zero_page_store_reg_x() {
        // LDX #$05; STX $0F => Store X Register in Memory
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x86, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x05);
        assert_eq!(cpu.reg_x, 0x05);
    }

    // ---------- STY tests
    #[test]
    fn test_0x84_sty_zero_age_store_reg_y() {
        // LDY #$05; STY $0F => Store Y Register in Memory
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0x84, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x05);
        assert_eq!(cpu.reg_y, 0x05);
    }

    // ---------- DEC tests
    #[test]
    fn test_0xc6_dec_zero_page_normal_decrement() {
        // LDA #$05; STA $0F; DEC $0F => Decrement Memory
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x04);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc6_dec_zero_page_decrement_to_zero() {
        // LDA #$01; STA $0F; DEC $0F => Decrement Memory to Zero
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x00);
        assert_eq!(cpu.reg_a, 0x01);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xc6_dec_zero_page_decrement_to_negative() {
        // LDA #$00; STA $0F; DEC $0F => Decrement Memory to Negative
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x85, 0x0f, 0xc6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0xFF);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- DEX tests
    #[test]
    fn test_0xca_dex_normal_decrement() {
        // LDX #$05; DEX => Decrement X Register
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0xca, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xca_dex_decrement_to_zero() {
        // LDX #$01; DEX => Decrement X Register to Zero
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x01, 0xca, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xca_dex_decrement_to_negative() {
        // LDX #$00; DEX => Decrement X Register to Negative
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x00, 0xca, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- EOR tests
    #[test]
    fn test_0x49_eor_normal_exclusive_or() {
        // LDA #$AA; EOR #$F5 => Exclusive OR Accumulator with Memory
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xaa, 0x49, 0xF5, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x5F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x49_eor_normal_exclusive_or_zero() {
        // LDA #$FF; EOR #$FF => Exclusive OR Accumulator with Memory (Zero Result)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x49, 0xFF, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x49_eor_normal_exclusive_or_negative() {
        // LDA #$7F; EOR #$FF => Exclusive OR Accumulator with Memory (Negative Result)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7F, 0x49, 0xFF, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- INC tests
    #[test]
    fn test_0xe6_inc_zero_page_normal_increment() {
        // LDA #$05; STA $0F; INC $0F => Increment Memory
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x05, 0x85, 0x0f, 0xe6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x06);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe6_inc_zero_page_increment_to_zero() {
        // LDA #$FF; STA $0F; INC $0F => Increment Memory to Zero
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0x85, 0x0f, 0xe6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xe6_inc_zero_page_increment_to_negative() {
        // LDA #$7F; STA $0F; INC $0F => Increment Memory to Negative
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7F, 0x85, 0x0f, 0xe6, 0x0f, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0f), 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- INX tests
    #[test]
    fn test_0xe8_inx_normal_increment() {
        // LDA #$20; TAX; INX => Increment X Register
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x20, 0xaa, 0xe8, 0x00]);
        cpu.run();
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_x, 33);
    }

    #[test]
    fn test_0xe8_inx_increment_into_negative() {
        // LDA #$7F; TAX; INX => Increment X Register into Negative
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x7F, 0xaa, 0xe8, 0x00]);
        cpu.run();
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    #[test]
    fn test_0xe8_inx_increment_into_zero() {
        // LDA #$FF; TAX; INX => Increment X Register into Zero
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xFF, 0xaa, 0xe8, 0x00]);
        cpu.run();
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_x, 0);
    }

    // ---------- INY tests
    #[test]
    fn test_0xc8_iny_normal_increment() {
        // LDY #$20; INY => Increment Y Register
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x20, 0xc8, 0x00]);
        cpu.run();
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 33);
    }

    #[test]
    fn test_0xc8_iny_increment_into_negative() {
        // LDY #$7F; INY => Increment Y Register into Negative
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x7F, 0xc8, 0x00]);
        cpu.run();
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 0x80);
    }

    #[test]
    fn test_0xc8_iny_increment_into_zero() {
        // LDY #$FF; INY => Increment Y Register into Zero
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0xFF, 0xc8, 0x00]);
        cpu.run();
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_zero_flag_set());
        assert_eq!(cpu.reg_y, 0);
    }

    // ---------- JMP tests
    #[test]
    fn test_0x4c_jmp_normal_jump() {
        // JMP $1000 => Jump to Address
        let mut cpu = convert_program_to_cpu(vec![0x4c, 0x00, 0x10]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x1001);
    }

    #[test]
    fn test_0x6c_jmp_normal_jump_indirect() {
        // JMP ($1000) => Jump to Address (Indirect)
        let mut cpu = convert_program_to_cpu(vec![0x6c, 0x00, 0x10]);
        cpu.mem_write(0x1000, 0x00);
        cpu.mem_write(0x1001, 0x20);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x2001);
    }

    #[test]
    fn test_0x6c_jmp_normal_jump_indirect_boundary_bug() {
        // Instead of loading addresses at $10FF and $1100 where 10FF is the lowbyte and 1100 is the high byte
        // It's loading the low byte from $10ff and the high byte from $1000 therefore returning the address $1050
        // JMP ($10FF) => Jump to Address (Indirect with Boundary Bug)
        let mut cpu = convert_program_to_cpu(vec![0x6c, 0xFF, 0x10]);
        cpu.mem_write(0x1051, 0x20);
        cpu.mem_write(0x1000, 0x10);
        cpu.mem_write(0x10FF, 0x50);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x1051);
    }

    // ---------- JSR tests
    #[test]
    fn test_0x20_jsr_normal_jump() {
        // Program counter starts at 0x8000, we read 3 bytes so its 8003 when we finish the JSR command but we push the pc minus 1
        // Therefore we should see the high byte 0x80 in the stack followed by the low byte 0x02
        // The program counter should also be at 10001

        // JSR $1000 => Jump to Subroutine
        let mut cpu = convert_program_to_cpu(vec![0x20, 0x00, 0x10]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01FD), 0x80);
        assert_eq!(cpu.mem_read(0x01FC), 0x02);
        assert_eq!(cpu.program_counter, 0x1001);
    }

    // ---------- LSR tests
    #[test]
    fn test_0x4a_lsr_accumulator() {
        // LDA #$02; LSR => Logical Shift Right Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x02, 0x4a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x46_lsr_zero_page() {
        // LSR $05 => Logical Shift Right Memory
        let mut cpu = convert_program_to_cpu(vec![0x46, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x02);
        cpu.run();
        assert_eq!(cpu.mem_read(0x05), 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x4a_lsr_carry_flag() {
        // LDA #$81; LSR => Logical Shift Right Accumulator (Carry Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x81, 0x4a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x40);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x4a_lsr_zero_flag() {
        // LDA #$01; LSR => Logical Shift Right Accumulator (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0x4a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }
    
    // ---------- NOP tests
    #[test]
    fn test_0xea_nop() {
        // NOP => No Operation
        let mut cpu = convert_program_to_cpu(vec![0xEA, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8002);
    }

    // ---------- ORA tests
    #[test]
    fn test_0x09_ora_normal_inclusive_or() {
        // LDA #$2A; ORA #$75 => Inclusive OR Accumulator with Memory
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x2a, 0x09, 0x75, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x7F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x09_ora_normal_inclusive_or_zero() {
        // LDA #$00; ORA #$00 => Inclusive OR Accumulator with Memory (Zero Result)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0x09, 0x00, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x09_ora_normal_inclusive_or_negative() {
        // LDA #$AA; ORA #$0F => Inclusive OR Accumulator with Memory (Negative Result)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0xAA, 0x09, 0x0F, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0xAF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- PHA tests
    #[test]
    fn test_0x48_pha_push_accumulator() {
        // LDA #$80; PHA => Push Accumulator to Stack
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0x48, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01FD), 0x80);
    }

    // ---------- PHP tests
    #[test]
    fn test_0x09_php_push_status_flag() {
        // SEC; SED; SEI; PHP => Push Status Flag to Stack
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xf8, 0x78, 0x08, 0x00]);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01FD), 0x3D);
    }

    // ---------- PLA tests
    #[test]
    fn test_0x68_pla_pull_stack_into_accumulator() {
        // LDA #$80; PHA; LDA #$01; PLA => Pull Stack to Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0x48, 0xa9, 0x01, 0x68, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert_eq!(cpu.stack_ptr, 0xFD);
    }

    // ---------- PLP tests
    #[test]
    fn test_0x28_plp_pull_status_flag() {
        // SEC; SED; SEI; PHP; CLC; CLD; CLI; PLP => Pull Status Flag from Stack
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xf8, 0x78, 0x08, 0x18, 0xD8, 0x58, 0x28, 0x00]);
        cpu.run();
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_decimal_flag_set());
        assert!(cpu.is_interrupt_flag_set());
    }

    // ---------- ROL tests
    #[test]
    fn test_0x2a_rol_accumulator() {
        // LDA #$02; ROL => Rotate Left Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x02, 0x2a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x26_rol_zero_page() {
        // ROL $05 => Rotate Left Memory
        let mut cpu = convert_program_to_cpu(vec![0x26, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x02);
        cpu.run();
        assert_eq!(cpu.mem_read(0x05), 0x04);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_bit_7_to_carry_flag() {
        // LDA #$81; ROL => Rotate Left Accumulator (Carry Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x81, 0x2a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x02);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_carry_flag_to_bit_0() {
        // LDA #$01; SEC; ROL => Rotate Left Accumulator (Carry Flag to Bit 0)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0x38, 0x2a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x03);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_zero_flag() {
        // LDA #$80; ROL => Rotate Left Accumulator (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0x2a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x2a_rol_negative_flag() {
        // LDA #$40; ROL => Rotate Left Accumulator (Negative Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x40, 0x2a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    // ---------- ROR tests
    #[test]
    fn test_0x6a_ror_accumulator() {
        // LDA #$02; ROR => Rotate Right Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x02, 0x6a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x66_ror_zero_page() {
        // ROR $05 => Rotate Right Memory
        let mut cpu = convert_program_to_cpu(vec![0x66, 0x05, 0x00]);
        cpu.mem_write(0x05, 0x02);
        cpu.run();
        assert_eq!(cpu.mem_read(0x05), 0x01);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_bit_0_to_carry_flag() {
        // LDA #$81; ROR => Rotate Right Accumulator (Carry Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x81, 0x6a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x40);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_carry_flag_to_bit_7_and_negative() {
        // LDA #$02; SEC; ROR => Rotate Right Accumulator (Carry Flag to Bit 7 and Negative)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x02, 0x38, 0x6a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x81);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0x6a_ror_zero_flag() {
        // LDA #$01; ROR => Rotate Right Accumulator (Zero Flag Set)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x01, 0x6a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    // ---------- RTI tests
    #[test]
    fn test_0x40_rti_push_status_flag_and_push_acc_then_rti() {
        // Since I don't currenty have a way to test interupts I'm going to make a pretend one by pushing the accumulator twice
        // and having that be the "program counter".
        // TODO: When interrupts get implemented update this unit test
        // LDA #$30; PHA; PHA; SEC; SED; SEI; PHP; CLC; CLD; CLI; RTI => Return from Interrupt
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x30, 0x48, 0x48, 0x38, 0xf8, 0x78, 0x08, 0x18, 0xD8, 0x58, 0x40, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x3031);
        assert!(cpu.is_carry_flag_set());
        assert!(cpu.is_decimal_flag_set());
        assert!(cpu.is_interrupt_flag_set());
    }

    // ---------- RTS tests
    #[test]
    fn test_0x60_rts_jump_then_return_from_jump() {
        // JSR $1000; RTS => Return from Subroutine
        let mut cpu = convert_program_to_cpu(vec![0x20, 0x00, 0x10, 0x00]);
        cpu.mem_write(0x1000, 0x60);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x8004);
    }

    // ---------- SBC tests
    #[test]
    fn test_0xe9_sbc_normal_subtract_no_borrow() {
        // SEC; LDA #$40; SBC #$05 => Subtract with Carry (No Borrow)
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xa9, 0x40, 0xe9, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x3b);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_normal_subtract_with_borrow() {
        // LDA #$40; SBC #$05 => Subtract with Carry (With Borrow)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x40, 0xe9, 0x05, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x3a);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_cause_no_carry_and_negative() {
        // This tests the case where a borrow occured and the carry flag is cleared to 0

        // SEC; LDA #$09; SBC #$0A => Subtract with Carry (No Carry and Negative)
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xa9, 0x09, 0xe9, 0x0a, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_subtract_positive_from_negative_for_overflow() {
        // SEC; LDA #$80; SBC #$01 => Subtract with Carry (Overflow from Positive to Negative)
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xa9, 0x80, 0xe9, 0x01, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x7F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_subtract_negative_from_positive_for_overflow() {
        // SEC; LDA #$7F; SBC #$FF => Subtract with Carry (Overflow from Negative to Positive)
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xa9, 0x7F, 0xe9, 0xFF, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(!cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_cause_zero() {
        // SEC; LDA #$01; SBC #$01 => Subtract with Carry (Zero Result)
        let mut cpu = convert_program_to_cpu(vec![0x38, 0xa9, 0x01, 0xe9, 0x01, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    #[test]
    fn test_0xe9_sbc_subtract_zero_with_carry_cause_overflow() {
        // CLC; LDA #$80; SBC #$00 => Subtract with Carry (Zero Result)
        let mut cpu = convert_program_to_cpu(vec![0x18, 0xa9, 0x80, 0xe9, 0x00, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x7F);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
        assert!(cpu.is_overflow_flag_set());
        assert!(cpu.is_carry_flag_set());
    }

    // ---------- TAY tests
    #[test]
    fn test_0xa8_tay_a_is_zero() {
        // LDA #$00; TAY => Transfer Accumulator to Y Register (Zero)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x00, 0xa8, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0xa8_tay_a_is_negative() {
        // LDA #$80; TAY => Transfer Accumulator to Y Register (Negative)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x80, 0xa8, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x80);
        assert!(cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    #[test]
    fn test_0xa8_tay_neither_negative_or_zero() {
        // LDA #$20; TAY => Transfer Accumulator to Y Register (Neither Negative nor Zero)
        let mut cpu = convert_program_to_cpu(vec![0xa9, 0x20, 0xa8, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_y, 0x20);
        assert!(!cpu.is_negative_flag_set());
        assert!(!cpu.is_zero_flag_set());
    }

    // ---------- TSX tests
    #[test]
    fn test_0xba_tsx_stack_is_fd() {
        // TSX => Transfer Stack Pointer to X Register (Stack Pointer is FF)
        let mut cpu = convert_program_to_cpu(vec![0xba, 0x00]);
        cpu.run();
        assert_eq!(cpu.reg_x, 0xFD);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- TXS tests
    #[test]
    fn test_0x9a_txs_x_standard() {
        // LDX #$05; TXS => Transfer X Register to Stack Pointer
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x9a, 0x00]);
        cpu.run();
        assert_eq!(cpu.stack_ptr, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x9a_txs_x_is_zero() {
        // TXS => Transfer X Register to Stack Pointer (X is Zero)
        let mut cpu = convert_program_to_cpu(vec![0x9a, 0x00]);
        cpu.run();
        assert_eq!(cpu.stack_ptr, 0x00);
    }

    #[test]
    fn test_0x9a_txs_x_is_negative() {
        // LDX #$80; TXS => Transfer X Register to Stack Pointer (X is Negative)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x80, 0x9a, 0x00]);
        cpu.run();
        assert_eq!(cpu.stack_ptr, 0x80);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- TXA tests
    #[test]
    fn test_0x8a_txa_standard() {
        // LDX #$05; TXA => Transfer X Register to Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x05, 0x8a]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x8a_txa_zero() {
        // LDX #$00; TXA => Transfer X Register to Accumulator (Zero)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0x00, 0x8a]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x8a_txa_negative() {
        // LDX #$FF; TXA => Transfer X Register to Accumulator (Negative)
        let mut cpu = convert_program_to_cpu(vec![0xa2, 0xFF, 0x8a]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }

    // ---------- TYA tests
    #[test]
    fn test_0x98_tya_standard() {
        // LDY #$05; TYA => Transfer Y Register to Accumulator
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x05, 0x98]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x05);
        assert!(!cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x98_tya_zero() {
        // LDY #$00; TYA => Transfer Y Register to Accumulator (Zero)
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0x00, 0x98]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.is_zero_flag_set());
        assert!(!cpu.is_negative_flag_set());
    }

    #[test]
    fn test_0x98_tya_negative() {
        // LDY #$FF; TYA => Transfer Y Register to Accumulator (Negative)
        let mut cpu = convert_program_to_cpu(vec![0xa0, 0xFF, 0x98]);
        cpu.run();
        assert_eq!(cpu.reg_a, 0xFF);
        assert!(!cpu.is_zero_flag_set());
        assert!(cpu.is_negative_flag_set());
    }
}