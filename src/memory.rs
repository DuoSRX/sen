use std;

pub struct Ram {
    pub val: [u8; 0xFFFF]//[u8; 0x800]
}

impl std::fmt::Debug for Ram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Ram {
    pub fn load(&self, address: u16) -> u8 {
        self.val[address as usize & 0x7ff]
    }

    pub fn store(&mut self, address: u16, value: u8) {
        self.val[address as usize & 0x7ff] = value;
    }
}
