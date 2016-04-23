use std;
use std::fs::File;
use std::io::prelude::*;
use std::fmt;

// Reference: http://wiki.nesdev.com/w/index.php/INES

#[allow(dead_code)]
pub struct NesHeader {
    magic: [u8; 4], // Magic number ("NES\x1a")
    pub prg_size: u8,   // PRG Rom banks (by increments of 16KB)
    chr_size: u8,   // CHR Rom banks (by increments of 8KB)
    flags_6: u8,
    flags_7: u8,
    ram_size: u8,   // PRG Ram size (by increments of 8KB)
    //unused: [u8; 7] // Unused stuff
}

pub struct Cartridge {
    pub header: NesHeader,
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub ram: [u8; 0x2000]
}

// TODO: Handle EOF errors
fn file_to_buffer(mut buffer: &mut [u8], file: &mut File) {
    let mut i = 0;
    while i < buffer.len() {
        let n = file.read(&mut buffer[i..]).unwrap();
        i += n
    }
}

impl Cartridge {
    pub fn load(mut file: &mut File) -> Cartridge {
        let mut header: [u8; 16] = [0; 16];
        file_to_buffer(&mut header, &mut file);

        let header = NesHeader {
            magic: [header[0], header[1], header[2], header[3]],
            prg_size: header[4],
            chr_size: header[5],
            flags_6: header[6],
            flags_7: header[7],
            ram_size: header[8]
        };

        let prg_len = header.prg_size as usize * 0x4000;
        let mut prg_rom = vec![0; prg_len];
        file_to_buffer(&mut prg_rom, &mut file);

        let chr_len = header.chr_size as usize * 0x2000;
        let mut chr_rom = vec![0; chr_len];
        file_to_buffer(&mut chr_rom, &mut file);

        Cartridge {
            header: header,
            prg: prg_rom,
            chr: chr_rom,
            ram: [0; 0x2000]
        }
    }
}

impl fmt::Display for NesHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PRG {}KB; CHR {}KB", self.prg_size as usize * 16, self.chr_size as usize * 8)
    }
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.header.ram_size as usize * 16))
    }
}
