// use std::error::Error;
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// TODO: Implement Absolute and Indirect addressing modes
// TODO: Implement a shitton more instructions
// TODO: Factor out the memory code (we'll need to handle mappers and such)

const CARRY_FLAG:     u8 = 0b00000001;
const ZERO_FLAG:      u8 = 0b00000010;
const INTERRUPT_FLAG: u8 = 0b00000100;
const DECIMAL_FLAG:   u8 = 0b00001000;
//const BREAK_FLAG:    u8 = 0b00010000;
const OVERFLOW_FLAG:  u8 = 0b01000000;
const NEGATIVE_FLAG:  u8 = 0b10000000;

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
            p: 0x24, // 0b00100100
            s: 0xFD, // 253
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
            // 0x00 => self.brk(), TODO
            0xEA => (), // NOP

            // Flag operations
            0x18 => self.clc(),
            0x38 => self.sec(),
            0x58 => self.cli(),
            0x78 => self.sei(),
            0xB8 => self.clv(),
            0xD8 => self.cld(),
            0xF8 => self.sed(),

            // 0x01 => { let am = self.indexed_indirect_x(); self.ora(am) }
            // 0x11 => { let am = self.indexed_indirect_y(); self.ora(am) }
            0x09 => self.ora(ImmediateAM),
            0x0D => { let am = self.absolute(); self.ora(am) }
            0x1D => { let am = self.absolute_x(); self.ora(am) }
            0x19 => { let am = self.absolute_y(); self.ora(am) }
            0x05 => { let am = self.zero_page(); self.ora(am) }
            0x15 => { let am = self.zero_page_x(); self.ora(am) }

            // Stack
            0x08 => self.php(),
            0x28 => self.plp(),
            0x48 => self.pha(),
            0x68 => self.pla(),

            // Register
            0xAA => self.tax(),
            0x8A => self.txa(),
            0xCA => self.dex(),
            0xE8 => self.inx(),
            0xA8 => self.tay(),
            0x98 => self.tya(),
            0x88 => self.dey(),
            0xC8 => self.iny(),

            0x0A => self.asl(AccumulatorAM),
            0x06 => { let am = self.zero_page(); self.asl(am) }
            0x16 => { let am = self.zero_page_x(); self.asl(am) }
            0x0E => { let am = self.absolute(); self.asl(am) }
            0x1E => { let am = self.absolute_x(); self.asl(am) }

            0xA9 => self.lda(ImmediateAM),
            0xAD => { let am = self.zero_page(); self.lda(am) }
            0xB5 => { let am = self.zero_page_x(); self.lda(am) }
            0xA2 => self.ldx(ImmediateAM),
            0xA0 => self.ldy(ImmediateAM),

            unknown => panic!("Unkown opcode {:02x}", unknown)
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
        let byte = self.load_byte(pc);
        self.regs.pc += 1;
        byte
    }

    fn load_word_and_inc_pc(&mut self) -> u16 {
        let a = self.load_byte_and_inc_pc() as u16;
        let b = self.load_byte_and_inc_pc() as u16;
        a | (b << 8)
    }

    fn push_byte(&mut self, value: u8) {
        let stack_pointer = self.regs.s;
        self.store_byte(0x100 + stack_pointer as u16, value);
        self.regs.s -= 1;
    }

    fn pop_byte(&mut self) -> u8 {
        let stack_pointer = self.regs.s;
        let byte = self.load_byte(0x100 + stack_pointer as u16);
        self.regs.s += 1;
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

    //fn zero_page_y(&mut self) -> MemoryAM {
    //    let address = self.load_byte_and_inc_pc() + self.regs.y;
    //    MemoryAM { address: address as u16 }
    //}

    fn absolute(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc();
        MemoryAM { address: address }
    }

    fn absolute_x(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc() + self.regs.x as u16;
        MemoryAM { address: address }
    }

    fn absolute_y(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc() + self.regs.y as u16;
        MemoryAM { address: address }
    }

    // FIXME
    /*fn indexed_indirect(&mut self) -> MemoryAM {
        let byte = self.load_byte_and_inc_pc();
        let indirect = byte + self.regs.x;
        let address = self.load_word()
        MemoryAM { address: address }
    }*/

    /*fn sta<AM: AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        am.store(self, a)
    }*/

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

    // Register
    fn tax(&mut self) {
        let a = self.regs.a;
        self.set_nz_flags(a);
        self.regs.x = a;
    }

    fn txa(&mut self) {
        let x = self.regs.x;
        self.set_nz_flags(x);
        self.regs.a = x;
    }

    fn dex(&mut self) {
        let x = self.regs.x - 1;
        self.regs.x = x;
        self.set_nz_flags(x);
    }

    fn inx(&mut self) {
        let x = self.regs.x + 1;
        self.regs.x = x;
        self.set_nz_flags(x);
    }

    fn tay(&mut self) {
        let a = self.regs.a;
        self.set_nz_flags(a);
        self.regs.y = a;
    }

    fn tya(&mut self) {
        let y = self.regs.y;
        self.set_nz_flags(y);
        self.regs.a = y;
    }

    fn dey(&mut self) {
        let y = self.regs.y - 1;
        self.regs.y = y;
        self.set_nz_flags(y);
    }

    fn iny(&mut self) {
        let y = self.regs.y + 1;
        self.regs.y = y;
        self.set_nz_flags(y);
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

    fn ora<AM:AddressingMode>(&mut self, am: AM) {
        let result = self.regs.a | am.load(self);
        self.set_nz_flags(result);
        self.regs.a = result;
    }

    // Stack operations
    fn pha(&mut self) {
        let a = self.regs.a;
        self.push_byte(a)
    }

    fn php(&mut self) {
        let a = self.regs.a;
        self.push_byte(a)
    }

    fn plp(&mut self) {
        self.regs.p = self.pop_byte()
    }

    fn pla(&mut self) {
        self.regs.a = self.pop_byte()
    }

    // Flags operations
    fn clc(&mut self) {
        self.unset_flag(CARRY_FLAG);
    }

    fn sec(&mut self) {
        self.set_flag(CARRY_FLAG);
    }

    fn cli(&mut self) {
        self.unset_flag(INTERRUPT_FLAG);
    }

    fn sei(&mut self) {
        self.set_flag(INTERRUPT_FLAG);
    }

    fn clv(&mut self) {
        self.unset_flag(OVERFLOW_FLAG);
    }

    fn cld(&mut self) {
        self.unset_flag(DECIMAL_FLAG);
    }

    fn sed(&mut self) {
        self.set_flag(DECIMAL_FLAG);
    }
}

fn main() {
    let mut cpu = Cpu::new();
    println!("{:?}", cpu);
    println!("Flags: {:08b}", cpu.regs.p);
    cpu.ram.val[0] = 0x08;
    cpu.ram.val[1] = 0x68;

    cpu.step();
    println!("{:?}", cpu);
    println!("Flags: {:08b}", cpu.regs.p);

    cpu.step();
    println!("{:?}", cpu);
    println!("Flags: {:08b}", cpu.regs.p);
}
