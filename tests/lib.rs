extern crate sen;

#[cfg(test)]
use std::fs::File;
use std::path::Path;
use sen::cpu::Cpu;
use sen::ppu::Ppu;
use sen::cartridge::Cartridge;

fn make_cpu() -> Cpu {
    // TODO: Mock the cartridge and PPU
    let path = Path::new("/Users/xavier/code/rust/sen/roms/donkeykong.nes");
    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);

    let ppu = Ppu::new();
    let mut cpu = Cpu::new(cartridge, ppu);
    cpu.reset();
    cpu.regs.pc = 0x0100;
    cpu
}

#[test]
fn lda_immediate() {
    let mut cpu = make_cpu();

    cpu.store_byte(0x0100, 0xa9);
    cpu.store_byte(0x0101, 0xff);
    cpu.step();

    assert_eq!(0xFF, cpu.regs.a);
}

#[test]
fn sta_absolute() {
    let mut cpu = make_cpu();

    cpu.regs.a = 0xf9;
    cpu.store_byte(0x0100, 0x8d);
    cpu.store_word(0x0101, 0x1234);
    cpu.step();

    assert_eq!(0xf9, cpu.load_word(0x1234));
}

#[test]
fn lda_zero_page() {
    let mut cpu = make_cpu();

    cpu.store_byte(0x0100, 0xa5);
    cpu.store_byte(0x0101, 0x88);
    cpu.store_byte(0x88, 0xf9);
    cpu.step();

    assert_eq!(0xf9, cpu.regs.a);
}
