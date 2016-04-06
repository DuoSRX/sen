// use std::error::Error;
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// TODO: Implement Absolute and Indirect addressing modes
// TODO: Implement a shitton more instructions
// TODO: Factor out the memory code (we'll need to handle mappers and such)

const CARRY_FLAG:    u8 = 0b00000001;
const ZERO_FLAG:     u8 = 0b00000010;
const IRA_FLAG:      u8 = 0b00000100;
const DECIMAL_FLAG:  u8 = 0b00001000;
const BREAK_FLAG:    u8 = 0b00010000;
const OVERFLOW_FLAG: u8 = 0b01000000;
const NEGATIVE_FLAG: u8 = 0b10000000;

#[allow(dead_code)]
#[derive(Debug)]
struct Registers {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    pc: u16
}

impl Registers {
    fn new() -> Registers {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc: 0,
        }
    }
}

// The addressing mode trait was liberally inspired by sprocketnes
trait AddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8;
    fn store(&self, cpu: &mut Cpu, value: u8);
}

struct AccumulatorAM;
impl AddressingMode for AccumulatorAM {
    fn load(&self, cpu: &mut Cpu) -> u8 { cpu.regs.a }
    fn store(&self, cpu: &mut Cpu, value: u8) { cpu.regs.a = value }
}

struct MemoryAM {
    address: u16
}
impl AddressingMode for MemoryAM {
    fn load(&self, cpu: &mut Cpu) -> u8 { cpu.load_byte(self.address) }
    fn store(&self, cpu: &mut Cpu, value: u8) { cpu.store_byte(self.address, value) }
}

struct ImmediateAM;
impl AddressingMode for ImmediateAM {
    fn load(&self, cpu: &mut Cpu) -> u8 { cpu.load_byte_and_inc_pc() }
    fn store(&self, _cpu: &mut Cpu, _value: u8) { panic!("uhhh I can't store using Immediate Addressing Mode") }
}

struct Ram {
    pub val: [u8; 0x800]
}

impl std::fmt::Debug for Ram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Ram {
    fn load(&self, address: u16) -> u8 {
        self.val[address as usize & 0x7ff]
    }

    fn store(&mut self, address: u16, value: u8) {
        self.val[address as usize & 0x7ff] = value;
    }
}

#[derive(Debug)]
struct Cpu {
    regs: Registers,
    ram: Ram,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            ram: Ram {
                val: [0; 0x800]
            }
        }
    }

    fn step(&mut self) {
        let instruction = self.load_byte_and_inc_pc();
        self.execute_instruction(instruction);
        // TODO: Handle cycle count
    }

    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            // 0x00 => self.brk(),
            //0x01 => { self.indexed_indirect_x(); self.ora() }
            // 0x05 => { self.zero_page(); self.ora() }
            // 0x06 => { self.zero_page(); self.asl() }
            // 0x08 => self.php(),
            // 0x09 => { self.immediate(); self.ora() }
            // 0x0D => { self.absolute(); self.ora() }

            0x0A => self.asl(AccumulatorAM),
            0x06 => { let am = self.zero_page(); self.asl(am) }
            0x16 => { let am = self.zero_page_x(); self.asl(am) }
            //0x0E => { let am = self.absolute(); self.asl(am) }
            //0x1E => { let am = self.absolute_x(); self.asl(am) }

            0xEA => (), // NOP
            0xAA => self.tax(),
            0xA8 => self.tay(),
            0x8A => self.txa(),
            0x9A => self.tya(),
            0xE8 => self.inx(),
            0xA9 => self.lda(ImmediateAM),
            0xAD => { let am = self.zero_page(); self.lda(am) }
            0xB5 => { let am = self.zero_page_x(); self.lda(am) }
            0xA2 => self.ldx(ImmediateAM),
            0xA0 => self.ldy(ImmediateAM),
            _ => () //println!("Unknown opcode")
        }
    }

    fn load_byte(&mut self, address: u16) -> u8 {
        self.ram.load(address)
    }

    fn store_byte(&mut self, address: u16, value: u8) {
        self.ram.store(address, value)
    }

    fn load_byte_and_inc_pc(&mut self) -> u8 {
        let pc = self.regs.pc;
        let byte = self.ram.load(pc);
        self.regs.pc += 1;
        byte
    }

    fn set_nz_flags(&mut self, value: u8) {
        if value == 0 {
            self.set_flag(ZERO_FLAG)
        } else {
            self.unset_flag(ZERO_FLAG)
        }

        if (value & 0x80) != 0 {
            self.set_flag(NEGATIVE_FLAG)
        } else {
            self.unset_flag(NEGATIVE_FLAG)
        }
    }

    fn set_flag(&mut self, flag: u8) {
        self.regs.p |= flag;
    }

    fn unset_flag(&mut self, flag: u8) {
        self.regs.p &= !flag;
    }

    // Generate addressing modes
    fn zero_page(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc();
        MemoryAM { address: address as u16 }
    }

    fn zero_page_x(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc() + self.regs.x;
        MemoryAM { address: address as u16 }
    }

    fn zero_page_y(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc() + self.regs.y;
        MemoryAM { address: address as u16 }
    }

    fn sta<AM: AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        am.store(self, a)
    }

    fn lda<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.regs.a = val;
        self.set_nz_flags(val)
    }

    fn ldx<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.regs.x = val;
        self.set_nz_flags(val)
    }

    fn ldy<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.regs.y = val;
        self.set_nz_flags(val)
    }

    // Accumulator - Storage
    fn tax(&mut self) {
        let a = self.regs.a;
        self.set_nz_flags(a);
        self.regs.x = a;
    }

    fn tay(&mut self) {
        let a = self.regs.a;
        self.set_nz_flags(a);
        self.regs.y = a;
    }

    fn txa(&mut self) {
        let x = self.regs.x;
        self.set_nz_flags(x);
        self.regs.a = x;
    }

    fn tya(&mut self) {
        let y = self.regs.y;
        self.set_nz_flags(y);
        self.regs.a = y;
    }

    // Acummulator - Bitwise logic
    // Index registers X - Arithmetic
    fn inx(&mut self) {
        let x = self.regs.x + 1;
        self.regs.x = x;
        self.set_nz_flags(x);
    }

    fn asl<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        if (value & 0x80) != 0 {
            self.set_flag(CARRY_FLAG);
        }
        let result = value << 1;
        self.set_nz_flags(result);
        am.store(self, result)
    }
}

fn main() {
    let mut cpu = Cpu::new();
    cpu.ram.val[0] = 0x0A;
    cpu.regs.a = 129;
    // cpu.ram.val[0] = 0xA9;
    // cpu.ram.val[1] = 185;
    cpu.step();

    println!("{:?}", cpu);
    println!("Flags: {:08b}", cpu.regs.p);
}
