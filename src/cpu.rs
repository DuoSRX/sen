use std;

use memory::CpuMemory;

/*
 * CPU Memory Map (http://wiki.nesdev.com/w/index.php/CPU_memory_map)
 * Address     Size  Note
 * $0000-$07FF $0800 Internal RAM
 * $0800-$0FFF $0800 Mirrors of $0000-$07FF
 * $1000-$17FF $0800 Same as above
 * $1800-$1FFF $0800 Same as above
 * $2000-$2007 $0008 PPU registers
 * $2008-$3FFF $1FF8 Mirrors of $2000-2007 every 8 bytes
 * $4000-$401F $0020 APU and I/O registers
 * $4020-$FFFF $BFE0 Cartridge (PRG ROM/RAM/Mapper)
 *
 * The ROM is usually stored in $8000-$FFFF
 *
 * $FFFA-$FFFB NMI vector
 * $FFFC-$FFFD Reset vector
 * $FFFE-$FFFF IRQ vector
 */

pub const CARRY_FLAG:     u8 = 0b00000001;
pub const ZERO_FLAG:      u8 = 0b00000010;
pub const INTERRUPT_FLAG: u8 = 0b00000100;
pub const DECIMAL_FLAG:   u8 = 0b00001000;
pub const BREAK_FLAG:     u8 = 0b00010000;
pub const OVERFLOW_FLAG:  u8 = 0b01000000;
pub const NEGATIVE_FLAG:  u8 = 0b10000000;

// Lifted from FCEUX
// This is fine but doesn't include the added cycle when crossing a page
const CYCLES_PER_INSTRUCTION: [u8; 256] = [
    7,6,2,8,3,3,5,5,3,2,2,2,4,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    6,6,2,8,3,3,5,5,4,2,2,2,4,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    6,6,2,8,3,3,5,5,3,2,2,2,3,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    6,6,2,8,3,3,5,5,4,2,2,2,5,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    2,6,2,6,3,3,3,3,2,2,2,2,4,4,4,4,
    2,6,2,6,4,4,4,4,2,5,2,5,5,5,5,5,
    2,6,2,6,3,3,3,3,2,2,2,2,4,4,4,4,
    2,5,2,5,4,4,4,4,2,4,2,4,4,4,4,4,
    2,6,2,8,3,3,5,5,2,2,2,2,4,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    2,6,3,8,3,3,5,5,2,2,2,2,4,4,6,6,
    2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
];

// The addressing mode trait was liberally inspired by https://github.com/pcwalton/sprocketnes
trait AddressingMode {
    fn load(&self, cpu: &mut Cpu) -> u8;
    fn store(&self, cpu: &mut Cpu, value: u8);
}

struct AccumulatorAM;
impl AddressingMode for AccumulatorAM {
    fn load(&self, cpu: &mut Cpu) -> u8 { cpu.a }
    fn store(&self, cpu: &mut Cpu, value: u8) { cpu.a = value }
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

pub struct Cpu {
    pub ram: CpuMemory,
    pub cycle: u64,

    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub pc: u16,

    // flags
    carry: bool,
    zero: bool,
    interrupt: bool,
    brk: bool,
    decimal: bool,
    overflow: bool,
    sign: bool
}

impl Cpu {
    pub fn new(memory: CpuMemory) -> Cpu {
        Cpu {
            ram: memory,
            cycle: 0,

            a: 0,
            x: 0,
            y: 0,
            s: 0xFD, // 253
            pc: 0,

            // flags
            carry: false,
            zero: false,
            interrupt: false,
            brk: false,
            decimal: false,
            overflow: false,
            sign: false
        }
    }

    pub fn reset(&mut self) {
        let start = self.load_word(0xFFFC);
        println!("Starting at {:04x}", start);
        self.pc = start;
    }

    pub fn step(&mut self) {
        loop {
            let instruction = self.load_byte_and_inc_pc();
            // print!("{:04x}: {:?}", self.pc - 1 - 0xc79e, self);
            // print!("{:04x}: {:?}", self.pc - 1, self);
            // print!(" Flags: {:08b}", self.p);
            // println!(" Instruction: {:02x}", instruction);
            // let pc = self.pc;
            // println!(" Instr {:02x} {:02x} {:02x}", instruction, self.load_byte(pc), self.load_byte(pc + 1));
            // for i in 0..31 {
            //     print!("{:02x} ", self.ram.ram.val[i]);
            // }
            // println!("");

            self.execute_instruction(instruction);
            // TODO: Handle actual cycle count
            self.cycle += CYCLES_PER_INSTRUCTION[instruction as usize] as u64;
            if self.cycle > 113 { break; };
        }
    }

    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            // NOPs (all illegal except 0xEA)
            0xEA => (),
            0x1A => (),
            0x3A => (),
            0x5A => (),
            0x7A => (),
            0xDA => (),
            0xFA => (),
            0x04 => self.nop(1),
            0x0C => self.nop(1),
            0x44 => self.nop(1),
            0x64 => self.nop(1),
            0x14 => self.nop(2),
            0x80 => self.nop(2),
            0x1C => self.nop(2),
            0x3C => self.nop(2),
            0x5C => self.nop(2),
            0x7C => self.nop(2),
            0xDC => self.nop(2),
            0xFC => self.nop(2),

            // Registers
            0xAA => self.tax(),
            0x8A => self.txa(),
            0xCA => self.dex(),
            0xE8 => self.inx(),
            0xA8 => self.tay(),
            0x98 => self.tya(),
            0x88 => self.dey(),
            0xC8 => self.iny(),
            0x9A => self.txs(),
            0xBA => self.tsx(),

            // Rotations
            0x2A => self.rol(AccumulatorAM),
            0x26 => { let am = self.zero_page(); self.rol(am) }
            0x36 => { let am = self.zero_page_x(); self.rol(am) }
            0x2E => { let am = self.absolute(); self.rol(am) }
            0x3E => { let am = self.absolute_x(); self.rol(am) }

            0x6A => self.ror(AccumulatorAM),
            0x66 => { let am = self.zero_page(); self.rol(am) }
            0x76 => { let am = self.zero_page_x(); self.rol(am) }
            0x6E => { let am = self.absolute(); self.rol(am) }
            0x7E => { let am = self.absolute_x(); self.rol(am) }

            0x0A => self.asl(AccumulatorAM),
            0x06 => { let am = self.zero_page(); self.asl(am) }
            0x16 => { let am = self.zero_page_x(); self.asl(am) }
            0x0E => { let am = self.absolute(); self.asl(am) }
            0x1E => { let am = self.absolute_x(); self.asl(am) }

            0x4A => self.lsr(AccumulatorAM),
            0x46 => { let am = self.zero_page(); self.lsr(am) }
            0x56 => { let am = self.zero_page_x(); self.lsr(am) }
            0x4E => { let am = self.absolute(); self.lsr(am) }
            0x5E => { let am = self.absolute_x(); self.lsr(am) }

            // Bitwise
            0x29 => self.and(ImmediateAM),
            0x25 => { let am = self.zero_page(); self.and(am) }
            0x35 => { let am = self.zero_page_x(); self.and(am) }
            0x2D => { let am = self.absolute(); self.and(am) }
            0x3D => { let am = self.absolute_x(); self.and(am) }
            0x39 => { let am = self.absolute_y(); self.and(am) }
            0x21 => { let am = self.indirect_x(); self.and(am) }
            0x31 => { let am = self.indirect_y(); self.and(am) }

            0x49 => self.eor(ImmediateAM),
            0x45 => { let am = self.zero_page(); self.eor(am) }
            0x55 => { let am = self.zero_page_x(); self.eor(am) }
            0x4D => { let am = self.absolute(); self.eor(am) }
            0x5D => { let am = self.absolute_x(); self.eor(am) }
            0x59 => { let am = self.absolute_y(); self.eor(am) }
            0x41 => { let am = self.indirect_x(); self.eor(am) }
            0x51 => { let am = self.indirect_y(); self.eor(am) }

            0x09 => self.ora(ImmediateAM),
            0x0D => { let am = self.absolute(); self.ora(am) }
            0x1D => { let am = self.absolute_x(); self.ora(am) }
            0x19 => { let am = self.absolute_y(); self.ora(am) }
            0x05 => { let am = self.zero_page(); self.ora(am) }
            0x15 => { let am = self.zero_page_x(); self.ora(am) }
            0x01 => { let am = self.indirect_x(); self.ora(am) }
            0x11 => { let am = self.indirect_y(); self.ora(am) }

            // Flag operations
            0x18 => self.clc(),
            0x38 => self.sec(),
            0x58 => self.cli(),
            0x78 => self.sei(),
            0xB8 => self.clv(),
            0xD8 => self.cld(),
            0xF8 => self.sed(),

            // Branching
            0x10 => self.bpl(),
            0x30 => self.bmi(),
            0x50 => self.bvc(),
            0x70 => self.bvs(),
            0x90 => self.bcc(),
            0xB0 => self.bcs(),
            0xD0 => self.bne(),
            0xF0 => self.beq(),

            // Comparisons
            0xC9 => self.cmp(ImmediateAM),
            0xC5 => { let am = self.zero_page(); self.cmp(am) }
            0xD5 => { let am = self.zero_page_x(); self.cmp(am) }
            0xCD => { let am = self.absolute(); self.cmp(am) }
            0xDD => { let am = self.absolute_x(); self.cmp(am) }
            0xD9 => { let am = self.absolute_y(); self.cmp(am) }
            0xC1 => { let am = self.indirect_x(); self.cmp(am) }
            0xD1 => { let am = self.indirect_y(); self.cmp(am) }

            0xE0 => self.cpx(ImmediateAM),
            0xE4 => { let am = self.zero_page(); self.cpx(am) }
            0xEC => { let am = self.absolute(); self.cpx(am) }

            0xC0 => self.cpy(ImmediateAM),
            0xC4 => { let am = self.zero_page(); self.cpy(am) }
            0xCC => { let am = self.absolute(); self.cpy(am) }

            0x24 => { let am = self.zero_page(); self.bit(am) }
            0x2C => { let am = self.absolute(); self.bit(am) }

            // Jumps
            0x4C => self.jmp(),
            0x6C => self.jmp_indirect(),
            0x20 => self.jsr(),

            // Stack
            0x08 => self.php(),
            0x28 => self.plp(),
            0x48 => self.pha(),
            0x68 => self.pla(),

            // Increment/Decrements
            0xC6 => { let am = self.zero_page(); self.dec(am) }
            0xD6 => { let am = self.zero_page_x(); self.dec(am) }
            0xCE => { let am = self.absolute(); self.dec(am) }
            0xDE => { let am = self.absolute_x(); self.dec(am) }
            0xE6 => { let am = self.zero_page(); self.inc(am) }
            0xF6 => { let am = self.zero_page_x(); self.inc(am) }
            0xEE => { let am = self.absolute(); self.inc(am) }
            0xFE => { let am = self.absolute_x(); self.inc(am) }

            // Arithmetic
            0x69 => self.adc(ImmediateAM),
            0x65 => { let am = self.zero_page(); self.adc(am) }
            0x75 => { let am = self.zero_page_x(); self.adc(am) }
            0x6D => { let am = self.absolute(); self.adc(am) }
            0x7D => { let am = self.absolute_x(); self.adc(am) }
            0x79 => { let am = self.absolute_y(); self.adc(am) }
            0x61 => { let am = self.indirect_x(); self.adc(am) }
            0x71 => { let am = self.indirect_y(); self.adc(am) }

            0xE9 => self.sbc(ImmediateAM),
            0xE5 => { let am = self.zero_page(); self.sbc(am) }
            0xF5 => { let am = self.zero_page_x(); self.sbc(am) }
            0xED => { let am = self.absolute(); self.sbc(am) }
            0xFD => { let am = self.absolute_x(); self.sbc(am) }
            0xF9 => { let am = self.absolute_y(); self.sbc(am) }
            0xE1 => { let am = self.indirect_x(); self.sbc(am) }
            0xF1 => { let am = self.indirect_y(); self.sbc(am) }

            // Load
            0xA9 => self.lda(ImmediateAM),
            0xA5 => { let am = self.zero_page(); self.lda(am) }
            0xB5 => { let am = self.zero_page_x(); self.lda(am) }
            0xAD => { let am = self.absolute(); self.lda(am) }
            0xBD => { let am = self.absolute_x(); self.lda(am) }
            0xB9 => { let am = self.absolute_y(); self.lda(am) }
            0xA1 => { let am = self.indirect_x(); self.lda(am) }
            0xB1 => { let am = self.indirect_y(); self.lda(am) }

            0xA2 => self.ldx(ImmediateAM),
            0xA6 => { let am = self.zero_page(); self.ldx(am) }
            0xB6 => { let am = self.zero_page_y(); self.ldx(am) }
            0xAE => { let am = self.absolute(); self.ldx(am) }
            0xBE => { let am = self.absolute_y(); self.ldx(am) }

            0xA0 => self.ldy(ImmediateAM),
            0xA4 => { let am = self.zero_page(); self.ldy(am) }
            0xB4 => { let am = self.zero_page_x(); self.ldy(am) }
            0xAC => { let am = self.absolute(); self.ldy(am) }
            0xBC => { let am = self.absolute_x(); self.ldy(am) }

            // Store
            0x85 => { let am = self.zero_page(); self.sta(am) }
            0x95 => { let am = self.zero_page_x(); self.sta(am) }
            0x8D => { let am = self.absolute(); self.sta(am) }
            0x9D => { let am = self.absolute_x(); self.sta(am) }
            0x99 => { let am = self.absolute_y(); self.sta(am) }
            0x81 => { let am = self.indirect_x(); self.sta(am) }
            0x91 => { let am = self.indirect_y(); self.sta(am) }

            0x86 => { let am = self.zero_page(); self.stx(am) }
            0x96 => { let am = self.zero_page_y(); self.stx(am) }
            0x8E => { let am = self.absolute(); self.stx(am) }

            0x84 => { let am = self.zero_page(); self.sty(am) }
            0x94 => { let am = self.zero_page_x(); self.sty(am) }
            0x8C => { let am = self.absolute(); self.sty(am) }

            // Interrupt and misc
            0x00 => self.brk(),
            0x40 => self.rti(),
            0x60 => self.rts(),

            unknown => panic!("Unkown opcode {:02x}", unknown)
        }
    }

    pub fn load_byte(&mut self, address: u16) -> u8 {
        self.ram.load(address)
    }

    pub fn load_word(&mut self, address: u16) -> u16 {
        // TODO: Remove this?
        // let lo = self.ram.load(address) as u16;
        // let hi = self.ram.load(address + 1);
        // lo | hi << 8 self.loadb(addr) as u16 | (self.loadb(addr + 1) as u16) << 8
         self.ram.load(address) as u16 | (self.ram.load(address + 1) as u16) << 8
    }

    pub fn load_word_zero_page(&mut self, address: u16) -> u16 {
        // TODO: Remove this?
        // let lo = self.ram.load(address) as u16;
        // let hi = self.ram.load(address + 1) as u16;
        // lo | hi << 8
        self.ram.load(address as u16) as u16 | (self.ram.load((address + 1) as u16) as u16) << 8
    }

    pub fn store_byte(&mut self, address: u16, value: u8) {
        // Special case for DMA. Super ugly but ehhh...
        // TODO: Move this into the memory code
        if address == 0x4014 {
            self.cycle += 513;
            if self.cycle % 2 == 1 {
                self.cycle += 1;
            }
        }
        self.ram.store(address, value)
    }

    pub fn store_word(&mut self, address: u16, value: u16) {
        let lo = value & 0xFF;
        let hi = (value >> 8) & 0xFF;
        self.store_byte(address, lo as u8);
        self.store_byte(address + 1, hi as u8);
    }

    fn load_byte_and_inc_pc(&mut self) -> u8 {
        let pc = self.pc;
        let byte = self.load_byte(pc);
        self.pc += 1;
        byte
    }

    fn load_word_and_inc_pc(&mut self) -> u16 {
        let a = self.load_byte_and_inc_pc() as u16;
        let b = self.load_byte_and_inc_pc() as u16;
        a | b << 8
    }

    // Stack operations
    fn push_byte(&mut self, value: u8) {
        let stack_pointer = self.s;
        self.store_byte(0x100 + stack_pointer as u16, value);
        self.s = self.s.wrapping_sub(1);
    }

    fn push_word(&mut self, value: u16) {
        let stack_pointer = self.s.wrapping_sub(1);
        self.store_word(0x100 + stack_pointer as u16, value);
        self.s = self.s.wrapping_sub(2);
    }

    fn pop_byte(&mut self) -> u8 {
        let stack_pointer = self.s;
        let byte = self.load_byte(0x100 + stack_pointer as u16 + 1);
        self.s = self.s.wrapping_add(1);
        byte
    }

    fn pop_word(&mut self) -> u16 {
        let stack_pointer = self.s;
        let word = self.load_word(0x100 + stack_pointer as u16 + 1);
        self.s = self.s.wrapping_add(2);
        word
    }

    fn set_nz_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.sign = (value & 0x80) != 0;
    }

    // Generate addressing modes
    fn zero_page(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc();
        MemoryAM { address: address as u16 }
    }

    fn zero_page_x(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc().wrapping_add(self.x);
        MemoryAM { address: address as u16 }
    }

    fn zero_page_y(&mut self) -> MemoryAM {
        let address = self.load_byte_and_inc_pc().wrapping_add(self.y);
        MemoryAM { address: address as u16 }
    }

    fn absolute(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc();
        MemoryAM { address: address }
    }

    fn absolute_x(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc().wrapping_add(self.x as u16);
        MemoryAM { address: address }
    }

    fn absolute_y(&mut self) -> MemoryAM {
        let address = self.load_word_and_inc_pc().wrapping_add(self.y as u16);
        MemoryAM { address: address }
    }

    // e.g. LDA ($20,X)
    fn indirect_x(&mut self) -> MemoryAM {
        let target = self.load_byte_and_inc_pc().wrapping_add(self.x);
        let address = self.load_word_zero_page(target as u16);
        MemoryAM { address: address }
    }

    // e.g. LDA ($86),Y
    fn indirect_y(&mut self) -> MemoryAM {
        let target = self.load_byte_and_inc_pc();
        let address = self.load_word_zero_page(target as u16).wrapping_add(self.y as u16);
        MemoryAM { address: address }
    }

    fn get_flags(&self) -> u8 {
        let mut p = 0;

        if self.sign      { p |= NEGATIVE_FLAG }
        if self.overflow  { p |= OVERFLOW_FLAG }
        if self.brk       { p |= BREAK_FLAG }
        if self.decimal   { p |= DECIMAL_FLAG }
        if self.interrupt { p |= INTERRUPT_FLAG }
        if self.zero      { p |= ZERO_FLAG }
        if self.carry     { p |= CARRY_FLAG }

        return p
    }

    fn set_flags(&mut self, flags: u8) {
        self.sign = (flags & NEGATIVE_FLAG) != 0;
        self.overflow = (flags & OVERFLOW_FLAG) != 0;
        self.brk = (flags & BREAK_FLAG) != 0;
        self.decimal = (flags & DECIMAL_FLAG) != 0;
        self.interrupt = (flags & INTERRUPT_FLAG) != 0;
        self.zero = (flags & ZERO_FLAG) != 0;
        self.carry = (flags & CARRY_FLAG) != 0;
    }

    // Instructions
    fn nop(&mut self, to_skip: u8) {
        self.pc += to_skip as u16;
    }

    // Arithmetic
    fn adc<AM: AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let mut result = value as u32 + self.a as u32;
        if self.carry {
            result = result.wrapping_add(1);
        }

        self.carry = (result & 0x100) != 0;
        self.set_nz_flags(result as u8);

        let a = self.a;
        self.overflow = (a ^ value) & 0x80 == 0 && (a ^ result as u8) & 0x80 == 0x80;

        self.a = (result as u8) & 0xFF;
    }

    fn sbc<AM: AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let mut result = (value as u32).wrapping_sub(self.a as u32);
        if !self.carry {
            result = result.wrapping_sub(1);
        }

        self.carry = (result & 0x100) == 0;
        self.set_nz_flags(result as u8);

        let a = self.a;
        self.overflow = (a ^ value) & 0x80 == 0 && (a ^ result as u8) & 0x80 == 0x80;

        self.a = (result as u8) & 0xFF;
    }

    fn inc<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self).wrapping_add(1);
        self.set_nz_flags(val);
        am.store(self, val & 0xFF);
    }

    fn dec<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self).wrapping_sub(1);
        self.set_nz_flags(val);
        am.store(self, val & 0xFF);
    }

    fn lda<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.a = val;
        self.set_nz_flags(val)
    }

    fn ldx<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.x = val;
        self.set_nz_flags(val)
    }

    fn ldy<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self);
        self.y = val;
        self.set_nz_flags(val)
    }

    fn sta<AM: AddressingMode>(&mut self, am: AM) {
        let a = self.a;
        am.store(self, a)
    }

    fn stx<AM: AddressingMode>(&mut self, am: AM) {
        let x = self.x;
        am.store(self, x)
    }

    fn sty<AM: AddressingMode>(&mut self, am: AM) {
        let y = self.y;
        am.store(self, y)
    }

    // Register
    fn tax(&mut self) {
        let a = self.a;
        self.set_nz_flags(a);
        self.x = a;
    }

    fn txa(&mut self) {
        let x = self.x;
        self.set_nz_flags(x);
        self.a = x;
    }

    fn dex(&mut self) {
        let x = self.x.wrapping_sub(1);
        self.x = x;
        self.set_nz_flags(x);
    }

    fn inx(&mut self) {
        let x = self.x.wrapping_add(1);
        self.x = x;
        self.set_nz_flags(x);
    }

    fn tay(&mut self) {
        let a = self.a;
        self.set_nz_flags(a);
        self.y = a;
    }

    fn tya(&mut self) {
        let y = self.y;
        self.set_nz_flags(y);
        self.a = y;
    }

    fn dey(&mut self) {
        let y = self.y.wrapping_sub(1);
        self.y = y;
        self.set_nz_flags(y);
    }

    fn iny(&mut self) {
        let y = self.y.wrapping_add(1);
        self.y = y;
        self.set_nz_flags(y);
    }

    fn txs(&mut self) {
        self.s = self.x;
    }

    fn tsx(&mut self) {
        let s = self.s;
        self.x = s;
        self.set_nz_flags(s);
    }

    // FIXME: This seems overly complicated...
    fn rol<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let carry = (value & 0x80) != 0;
        let result = value << 1;

        if self.carry {
            am.store(self, result | 1);
        } else {
            am.store(self, result);
        }

        self.carry = carry;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    // FIXME: This seems overly complicated...
    fn ror<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let carry = (value & 1) != 0;
        let result = value >> 1;

        if self.carry {
            am.store(self, result | 0x80);
        } else {
            am.store(self, result);
        }

        self.carry = carry;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    fn and<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.a;
        let value = am.load(self) & a;
        self.set_nz_flags(value);
        self.a = value;
    }

    fn eor<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.a;
        let value = am.load(self) ^ a;
        self.set_nz_flags(value);
        self.a = value;
    }

    fn asl<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        self.carry = (value & 0x80) != 0;
        let result = value << 1;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    fn lsr<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        self.carry = (value & 1) != 0;
        let result = value >> 1;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    fn ora<AM:AddressingMode>(&mut self, am: AM) {
        let result = self.a | am.load(self);
        self.set_nz_flags(result);
        self.a = result;
    }

    // Jumps
    fn jmp(&mut self) {
         let address = self.load_word_and_inc_pc();
         self.pc = address;
    }

    fn jmp_indirect(&mut self) {
        // TODO: Implement the 6502 bug
        let indirect = self.load_word_and_inc_pc();

        let low = self.load_byte(indirect);
        let high = self.load_byte((indirect & 0xFF00) | ((indirect + 1) & 0x00FF));

        self.pc = ((high as u16) << 8) | low as u16;
    }

    fn jsr(&mut self) {
        let address = self.load_word_and_inc_pc();
        let pc = self.pc - 1;
        self.push_word(pc);
        self.pc = address;
    }

    // Stack operations
    fn pha(&mut self) {
        let a = self.a;
        self.push_byte(a)
    }

    fn php(&mut self) {
        let flags = self.get_flags() | BREAK_FLAG;
        self.push_byte(flags);
    }

    fn plp(&mut self) {
        let p = self.pop_byte();
        self.set_flags(p);
    }

    fn pla(&mut self) {
        self.a = self.pop_byte()
    }

    // Flags operations
    fn clc(&mut self) {
        self.carry = false;
    }

    fn sec(&mut self) {
        self.carry = true;
    }

    fn cli(&mut self) {
        self.interrupt = false;
    }

    fn sei(&mut self) {
        self.interrupt = true;
    }

    fn clv(&mut self) {
        self.overflow = false;
    }

    fn cld(&mut self) {
        self.decimal = false;
    }

    fn sed(&mut self) {
        self.decimal = true;
    }

    // Branching
    fn bpl(&mut self) {
        let sign = self.sign;
        self.generic_branching(!sign)
    }

    fn bmi(&mut self) {
        let sign = self.sign;
        self.generic_branching(sign)
    }

    fn bvc(&mut self) {
        let overflow = self.overflow;
        self.generic_branching(!overflow);
    }

    fn bvs(&mut self) {
        let overflow = self.overflow;
        self.generic_branching(overflow);
    }

    fn bcc(&mut self) {
        let carry = self.carry;
        self.generic_branching(!carry)
    }

    fn bcs(&mut self) {
        let carry = self.carry;
        self.generic_branching(carry)
    }

    fn bne(&mut self) {
        let zero = self.zero;
        self.generic_branching(!zero);
    }

    fn beq(&mut self) {
        let zero = self.zero;
        self.generic_branching(zero);
    }

    fn generic_branching(&mut self, go: bool) {
        let byte = self.load_byte_and_inc_pc() as i8;
        if go {
            self.pc = (self.pc as i32 + byte as i32) as u16;
        }
    }

    // Comparisons
    fn cmp<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.a;
        self.generic_comparison(am, a);
    }

    fn cpx<AM:AddressingMode>(&mut self, am: AM) {
        let x = self.x;
        self.generic_comparison(am, x);
    }

    fn cpy<AM:AddressingMode>(&mut self, am: AM) {
        let y = self.y;
        self.generic_comparison(am, y);
    }

    fn bit<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.a;
        let byte = am.load(self);
        let result = a & byte;
        let overflow = (result >> 6) & 1;
        self.overflow = overflow != 0;
        self.set_nz_flags(result);
    }

    fn generic_comparison<AM:AddressingMode>(&mut self, am: AM, reg: u8) {
        let byte = am.load(self);
        let value = reg.wrapping_sub(byte);
        self.set_nz_flags(value);
        self.carry = reg >= byte;
    }

    fn brk(&mut self) {
        let pc = self.pc;
        self.push_word(pc + 1);
        let flags = self.get_flags();
        self.push_byte(flags);
        self.sei();
        self.pc = self.load_word(0xFFFE);
    }

    fn rti(&mut self) {
        let flags = self.pop_byte();
        self.set_flags(flags);
        let pc = self.pop_word();
        self.pc = pc;
    }

    fn rts(&mut self) {
        let pc = self.pop_word();
        self.pc = pc + 1;
    }

    pub fn nmi(&mut self) {
        let pc = self.pc;
        self.push_word(pc);
        self.php();
        self.pc = self.load_word(0xFFFA);
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("A:{:02x} X:{:02x} Y:{:02x} Zero: {} SP:{:02X} PC:{:04x}",
            self.a,
            self.x,
            self.y,
            self.zero,
            self.get_flags(),
            self.pc
        ))
    }
}
