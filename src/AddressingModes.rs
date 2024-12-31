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
            AddressingMode::Indirect_X => {
                let addr: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let added_addr: u16 = addr.wrapping_add(cpu.reg_x as u16);
    
                cpu.mem_read_u16(added_addr)
            },
            AddressingMode::Indirect_Y => {
                let addr: u16 = cpu.mem_read(cpu.program_counter) as u16;
                let dereferenced_addr: u16 = cpu.mem_read_u16(addr);
                
                cpu.mem_read(dereferenced_addr.wrapping_add(cpu.reg_y as u16)) as u16
            },
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", self);
            }
        }
    }    
}