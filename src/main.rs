use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

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
            x: 2,
            y: 0,
            p: 0,
            s: 0,
            pc: 0,
        }
    }
}

// enum AddressingMode {
//     ImmediateAddressingMode,
//     MemoryAddressingMode,
//     AccumulatorAddressingMode(u16)
// }

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
    fn store(&self, cpu: &mut Cpu, value: u8) { panic!("uhhh I can't store using Immediate Addressing Mode") }
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
            ram: Ram { val: [0; 0x800] }
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
            self.regs.p |= 0x02;
        } else {
            self.regs.p &= 0xfd;
        }

        if (value & 0x80) != 0 {
            self.regs.p |= 0x80;
        } else {
            self.regs.p &= 0x7f;
        }
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

    // Index registers
    fn inx(&mut self) {
        let x = self.regs.x + 1;
        self.regs.x = x;
        self.set_nz_flags(x);
    }

    // Register Storage
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
}

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/missile1.h");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let mut buffer = Vec::new();

    match file.read_to_end(&mut buffer) {
        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
        Ok(_) => println!("{} is: {} bytes long", display, buffer.len()),
    }

    // let mut offset = 0;
    // for chunk in buffer.chunks(16) {
    //     print!("{:07x} ", offset);
    //     for b in chunk {
    //         print!("{:02x} ", b);
    //     }
    //     offset += 16;
    //     println!("");
    // }
    // println!("{:07x} ", offset);

    let mut i = 0;

    let mut cpu = Cpu::new();

    while i < buffer.len() {
        match buffer[i] {
            0xEA => (), // NOP
            0xAA => cpu.tax(),
            0xA8 => cpu.tay(),
            0x8A => cpu.txa(),
            0x9A => cpu.tya(),
            0xE8 => cpu.inx(),
            0xA9 => cpu.lda(ImmediateAM),
            0xA2 => cpu.ldx(ImmediateAM),
            0xA0 => cpu.ldy(ImmediateAM),
            _ => () //println!("Unknown opcode")
        }

        i += 1;
    }

    cpu.ram.val[0] = 123;
    cpu.lda(MemoryAM { address: 0 });

    println!("{:?}", cpu);
    //println!("{:08b}", cpu.regs.p);
}
