use crate::CPU;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

impl AddressingMode {
    pub fn get_operand_address(&self, cpu: &CPU) -> u16 {
        match self {
            AddressingMode::Immediate => cpu.program_counter,
            AddressingMode::ZeroPage => cpu.mem_read(cpu.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                let zero_page: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let x: u16 = cpu.reg_x as u16;
                zero_page.wrapping_add(x)
            },
            AddressingMode::ZeroPage_Y => {
                let zero_page: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let y: u16 = cpu.reg_y as u16;
                zero_page.wrapping_add(y)
            },
            AddressingMode::Absolute => cpu.mem_read_u16(cpu.program_counter),
            AddressingMode::Absolute_X => {
                let abs_addr: u16 = cpu.mem_read_u16(cpu.program_counter);
                abs_addr.wrapping_add(cpu.reg_x as u16)
            },
            AddressingMode::Absolute_Y => {
                let abs_addr: u16 = cpu.mem_read_u16(cpu.program_counter);
                abs_addr.wrapping_add(cpu.reg_y as u16)
            },
            AddressingMode::Indirect => {
                /* 
                    Indirect is only used by the JMP subroutine. And it has the following bug according to https://www.nesdev.org/obelisk-6502-guide/reference.html#JMP
                    
                    NB: An original 6502 has does not correctly fetch the target address if the indirect vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
                    In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
                    This is fixed in some later chips like the 65SC02 so for compatibility always ensure the indirect vector is not at the end of the page.

                    To maintain compatibility I will be coding that bug in as well 
                */
                
                let addr: u16 = cpu.mem_read_u16(cpu.program_counter);
                let low_byte: u16 = addr as u16 & 0b0000_0000_1111_1111;

                if (low_byte == 0xFF) { // Page boundary
                    let dereferenced_low_byte = cpu.mem_read(addr) as u16;
                    let dereferenced_high_byte = (cpu.mem_read(addr & 0b1111_1111_0000_0000) as u16) << 8;
                    return cpu.mem_read_u16(dereferenced_high_byte | dereferenced_low_byte);
                } else {
                    let dereferenced_addr: u16 = cpu.mem_read_u16(addr);
                    return dereferenced_addr;
                }

            },
            AddressingMode::Indirect_X => {
                let addr: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let added_addr: u16 = addr.wrapping_add(cpu.reg_x as u16);
    
                cpu.mem_read_u16(added_addr)
            },
            AddressingMode::Indirect_Y => {
                let addr: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let dereferenced_addr: u16 = cpu.mem_read_u16(addr);
                
                dereferenced_addr.wrapping_add(cpu.reg_y as u16) as u16
            },
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", self);
            }
        }
    }    
}