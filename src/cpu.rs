use memory::Ram;

const CARRY_FLAG:     u8 = 0b00000001;
const ZERO_FLAG:      u8 = 0b00000010;
const INTERRUPT_FLAG: u8 = 0b00000100;
const DECIMAL_FLAG:   u8 = 0b00001000;
//const BREAK_FLAG:    u8 = 0b00010000;
const OVERFLOW_FLAG:  u8 = 0b01000000;
const NEGATIVE_FLAG:  u8 = 0b10000000;

// The addressing mode trait was liberally inspired by https://github.com/pcwalton/sprocketnes
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
            pc: 0xC000,
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    regs: Registers,
    pub ram: Ram,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            ram: Ram {
                val: [0; 0xFFFF]
            }
        }
    }

    pub fn step(&mut self) {
        let instruction = self.load_byte_and_inc_pc();
        println!("{:?}", self);
        println!("Flags: {:08b}", self.regs.p);
        println!("Decoding: {:02x} at PC = {:02x}", instruction, self.regs.pc - 1);
        self.execute_instruction(instruction);
        // TODO: Handle cycle count
    }

    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            0xEA => (), // NOP

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
            0x2A => self.rol(ImmediateAM),
            0x26 => { let am = self.zero_page(); self.rol(am) }
            0x36 => { let am = self.zero_page_x(); self.rol(am) }
            0x2E => { let am = self.absolute(); self.rol(am) }
            0x3E => { let am = self.absolute_x(); self.rol(am) }

            0x6A => self.ror(ImmediateAM),
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
            0x21 => { let am = self.indexed_indirect(); self.and(am) }
            0x31 => { let am = self.indirect_indexed(); self.and(am) }

            0x49 => self.eor(ImmediateAM),
            0x45 => { let am = self.zero_page(); self.eor(am) }
            0x55 => { let am = self.zero_page_x(); self.eor(am) }
            0x4D => { let am = self.absolute(); self.eor(am) }
            0x5D => { let am = self.absolute_x(); self.eor(am) }
            0x59 => { let am = self.absolute_y(); self.eor(am) }
            0x41 => { let am = self.indexed_indirect(); self.eor(am) }
            0x51 => { let am = self.indirect_indexed(); self.eor(am) }

            0x09 => self.ora(ImmediateAM),
            0x0D => { let am = self.absolute(); self.ora(am) }
            0x1D => { let am = self.absolute_x(); self.ora(am) }
            0x19 => { let am = self.absolute_y(); self.ora(am) }
            0x05 => { let am = self.zero_page(); self.ora(am) }
            0x15 => { let am = self.zero_page_x(); self.ora(am) }
            0x01 => { let am = self.indexed_indirect(); self.ora(am) }
            0x11 => { let am = self.indirect_indexed(); self.ora(am) }

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
            0xC1 => { let am = self.indexed_indirect(); self.cmp(am) }
            0xD1 => { let am = self.indirect_indexed(); self.cmp(am) }

            0xE0 => self.cpx(ImmediateAM),
            0xE4 => { let am = self.zero_page(); self.cpx(am) }
            0xEC => { let am = self.absolute(); self.cpx(am) }

            0xC0 => self.cpy(ImmediateAM),
            0xC4 => { let am = self.zero_page(); self.cpy(am) }
            0xCC => { let am = self.absolute(); self.cpy(am) }

            // Jumps
            // 0x4C => { let am = self.absolute(); self.jmp(ImmediateAM) }
            // TODO: 0x6C => { let am = self.indirect(); self.jmp(am) }
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
            0xFF => { let am = self.absolute_x(); self.inc(am) }

            // Arithmetic
            0x69 => self.adc(ImmediateAM),
            0x65 => { let am = self.zero_page(); self.adc(am) }
            0x75 => { let am = self.zero_page_x(); self.adc(am) }
            0x6D => { let am = self.absolute(); self.adc(am) }
            0x7D => { let am = self.absolute_x(); self.adc(am) }
            0x79 => { let am = self.absolute_y(); self.adc(am) }
            0x61 => { let am = self.indexed_indirect(); self.adc(am) }
            0x71 => { let am = self.indirect_indexed(); self.adc(am) }

            0xE9 => self.sbc(ImmediateAM),
            0xE5 => { let am = self.zero_page(); self.sbc(am) }
            0xF5 => { let am = self.zero_page_x(); self.sbc(am) }
            0xED => { let am = self.absolute(); self.sbc(am) }
            0xFD => { let am = self.absolute_x(); self.sbc(am) }
            0xF9 => { let am = self.absolute_y(); self.sbc(am) }
            0xE1 => { let am = self.indexed_indirect(); self.sbc(am) }
            0xF1 => { let am = self.indirect_indexed(); self.sbc(am) }

            // Load
            0xA9 => self.lda(ImmediateAM),
            0xA5 => { let am = self.zero_page(); self.lda(am) }
            0xB5 => { let am = self.zero_page_x(); self.lda(am) }
            0xAD => { let am = self.absolute(); self.lda(am) }
            0xBD => { let am = self.absolute_x(); self.lda(am) }
            0xB9 => { let am = self.absolute_y(); self.lda(am) }
            0xA1 => { let am = self.indexed_indirect(); self.lda(am) }
            0xB1 => { let am = self.indirect_indexed(); self.lda(am) }

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
            0x81 => { let am = self.indirect_indexed(); self.sta(am) }
            0x91 => { let am = self.indexed_indirect(); self.sta(am) }

            0x86 => { let am = self.zero_page(); self.stx(am) }
            0x96 => { let am = self.zero_page_y(); self.stx(am) }
            0x8E => { let am = self.absolute(); self.stx(am) }

            0x84 => { let am = self.zero_page(); self.sty(am) }
            0x94 => { let am = self.zero_page_x(); self.sty(am) }
            0x8C => { let am = self.absolute(); self.sty(am) }

            // Interrupt and misc
            // TODO: RTI
            // TODO: RTS
            // TODO: BRK
            // TODO: BIT

            unknown => panic!("Unkown opcode {:02x}", unknown)
        }
    }

    fn load_byte(&mut self, address: u16) -> u8 {
        self.ram.load(address)
    }

    fn load_word(&mut self, address: u16) -> u16 {
        let lo = self.ram.load(address) as u16;
        let hi = self.ram.load(address + 1) as u16;
        lo | hi << 8
    }

    pub fn store_byte(&mut self, address: u16, value: u8) {
        self.ram.store(address, value)
    }

    fn store_word(&mut self, address: u16, value: u16) {
        let lo = value & 0xFF;
        let hi = (value >> 8) & 0xFF;
        self.store_byte(address, lo as u8);
        self.store_byte(address + 1, hi as u8);
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
        a | b << 8
    }

    fn push_byte(&mut self, value: u8) {
        let stack_pointer = self.regs.s;
        self.store_byte(0x100 + stack_pointer as u16, value);
        self.regs.s -= 1;
    }

    fn push_word(&mut self, value: u16) {
        let stack_pointer = self.regs.s - 1;
        self.store_word(0x100 + stack_pointer as u16, value);
        self.regs.s -= 2;
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

    fn check_flag(&mut self, flag: u8) -> bool {
        (self.regs.p & flag) != 0
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

    // e.g. LDA ($20,X)
    fn indexed_indirect(&mut self) -> MemoryAM {
        let page = self.load_byte_and_inc_pc();
        let indirect = page + self.regs.x;
        let address = self.load_word(indirect as u16);
        MemoryAM { address: address }
    }

    // e.g. LDA ($86),Y
    fn indirect_indexed(&mut self) -> MemoryAM {
        let page = self.load_byte_and_inc_pc();
        let indirect = page + self.regs.x;
        let address = self.load_word(indirect as u16);
        MemoryAM { address: address }
    }

    // Instructions
    // Arithmetic
    fn adc<AM: AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let mut result = value as u32 + self.regs.a as u32;
        if self.check_flag(CARRY_FLAG) {
            result += 1;
        }

        if (result & 0x100) != 0 {
            self.set_flag(CARRY_FLAG);
        } else {
            self.unset_flag(CARRY_FLAG);
        }

        self.set_nz_flags(result as u8);
        self.regs.a = (result as u8) & 0xFF;
        // TODO: Handle overflow flag
    }

    fn sbc<AM: AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let mut result = value as u32 - self.regs.a as u32;
        if !self.check_flag(CARRY_FLAG) {
            result -= 1;
        }

        if (result & 0x100) == 0 {
            self.set_flag(CARRY_FLAG);
        } else {
            self.unset_flag(CARRY_FLAG);
        }

        self.set_nz_flags(result as u8);
        self.regs.a = (result as u8) & 0xFF;
        // TODO: Handle overflow flag
    }

    fn inc<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self) + 1;
        self.set_nz_flags(val);
        am.store(self, val & 0xFF);
    }

    fn dec<AM: AddressingMode>(&mut self, am: AM) {
        let val = am.load(self) - 1;
        self.set_nz_flags(val);
        am.store(self, val & 0xFF);
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

    fn sta<AM: AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        am.store(self, a)
    }

    fn stx<AM: AddressingMode>(&mut self, am: AM) {
        let x = self.regs.x;
        am.store(self, x)
    }

    fn sty<AM: AddressingMode>(&mut self, am: AM) {
        let y = self.regs.y;
        am.store(self, y)
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

    fn txs(&mut self) {
        self.regs.s = self.regs.x;
    }

    fn tsx(&mut self) {
        let s = self.regs.s;
        self.regs.x = s;
        self.set_nz_flags(s);
    }

    // FIXME: This seems overly complicated...
    fn rol<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let carry = (value & 0x80) != 0;
        let result = value << 1;

        if self.check_flag(CARRY_FLAG) {
            am.store(self, result | 1);
        } else {
            am.store(self, result);
        }

        if carry {
            self.set_flag(CARRY_FLAG);
        } else {
            self.unset_flag(CARRY_FLAG);
        }

        self.set_nz_flags(result);
        am.store(self, result)
    }

    // FIXME: This seems overly complicated...
    fn ror<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        let carry = (value & 1) != 0;
        let result = value >> 1;

        if self.check_flag(CARRY_FLAG) {
            am.store(self, result | 0x80);
        } else {
            am.store(self, result);
        }

        if carry {
            self.set_flag(CARRY_FLAG);
        } else {
            self.unset_flag(CARRY_FLAG);
        }

        self.set_nz_flags(result);
        am.store(self, result)
    }

    fn and<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        let value = am.load(self) & a;
        self.set_nz_flags(value);
        self.regs.a = value;
    }

    fn eor<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        let value = am.load(self) ^ a;
        self.set_nz_flags(value);
        self.regs.a = value;
    }

    // FIXME: Set carry correctly
    fn asl<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        if (value & 0x80) != 0 {
            self.set_flag(CARRY_FLAG);
        }
        let result = value << 1;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    // FIXME: Set carry correctly
    fn lsr<AM:AddressingMode>(&mut self, am: AM) {
        let value = am.load(self);
        if (value & 1) != 0 {
            self.set_flag(CARRY_FLAG);
        }
        let result = value >> 1;
        self.set_nz_flags(result);
        am.store(self, result)
    }

    fn ora<AM:AddressingMode>(&mut self, am: AM) {
        let result = self.regs.a | am.load(self);
        self.set_nz_flags(result);
        self.regs.a = result;
    }

    // Jumps
    //fn jmp<AM:AddressingMode>(&mut self, am: AM) {
    // fn jmp<AM:AbsoluteAM>(&mut self, am: AM) {
    //     let address = am.load(self);
    //     self.regs.pc = address;
    // }

    fn jsr(&mut self) {
        let address = self.load_word_and_inc_pc();
        let pc = self.regs.pc - 1;
        self.push_word(pc);
        self.regs.pc = address;
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

    // Branching
    // TODO: All of these are super similar. Refactor that stuff!
    fn bpl(&mut self) {
        let plus = !self.check_flag(NEGATIVE_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if plus {
            self.regs.pc += byte as u16;
        }
    }

    fn bmi(&mut self) {
        let plus = self.check_flag(NEGATIVE_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if plus {
            self.regs.pc += byte as u16;
        }
    }

    fn bvc(&mut self) {
        let overflow = !self.check_flag(OVERFLOW_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if overflow {
            self.regs.pc += byte as u16;
        }
    }

    fn bvs(&mut self) {
        let overflow = self.check_flag(OVERFLOW_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if overflow {
            self.regs.pc += byte as u16;
        }
    }

    fn bcc(&mut self) {
        let carry = !self.check_flag(CARRY_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if carry {
            self.regs.pc += byte as u16;
        }
    }

    fn bcs(&mut self) {
        let carry = self.check_flag(CARRY_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if carry {
            self.regs.pc += byte as u16;
        }
    }

    fn bne(&mut self) {
        let zero = self.check_flag(ZERO_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if zero {
            self.regs.pc += byte as u16;
        }
    }

    fn beq(&mut self) {
        let zero = !self.check_flag(ZERO_FLAG);
        let byte = self.load_byte_and_inc_pc();
        if zero {
            self.regs.pc += byte as u16;
        }
    }

    // Comparisons
    fn cmp<AM:AddressingMode>(&mut self, am: AM) {
        let a = self.regs.a;
        self.generic_comparison(am, a);
    }

    fn cpx<AM:AddressingMode>(&mut self, am: AM) {
        let x = self.regs.x;
        self.generic_comparison(am, x);
    }

    fn cpy<AM:AddressingMode>(&mut self, am: AM) {
        let y = self.regs.y;
        self.generic_comparison(am, y);
    }

    fn generic_comparison<AM:AddressingMode>(&mut self, am: AM, reg: u8) {
        let byte = am.load(self);
        let value = reg - byte;

        if reg >= byte {
            self.set_flag(CARRY_FLAG);
        } else {
            self.unset_flag(CARRY_FLAG);
        }

        self.set_nz_flags(value);
    }
}
