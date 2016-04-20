pub struct Controller {
    pub buttons: [bool; 8],
    index: usize,
    strobe: u8
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            buttons: [false; 8],
            index: 0,
            strobe: 0
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        if address != 0x4016 { return 0 };
        let mut value = 0;

        if self.index < 8 && self.buttons[self.index] {
            value = 1;
        }

        self.index += 1;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }

        value
    }

    pub fn store(&mut self, address: u16, value: u8) {
        if address != 0x4016 { return };
        self.strobe = value;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }
}
