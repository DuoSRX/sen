// FIXME: This is completely broken ATM.

use sdl2::keyboard::Keycode;

pub struct Controller {
    // pub buttons: [bool; 8],
    pub buttons: Vec<Keycode>,
    index: u8,
    strobe: u8
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            // buttons: [false; 8],
            buttons: Vec::new(),
            index: 0,
            strobe: 0
        }
    }

    pub fn load(&mut self, _address: u16) -> u8 {
        let key = match self.index {
            1 => self.buttons.contains(&Keycode::Z), // A
            2 => self.buttons.contains(&Keycode::X), // B
            3 => self.buttons.contains(&Keycode::A), // Select
            4 => self.buttons.contains(&Keycode::Space), // Start
            5 => self.buttons.contains(&Keycode::Up),
            6 => self.buttons.contains(&Keycode::Down),
            7 => self.buttons.contains(&Keycode::Left),
            8 => self.buttons.contains(&Keycode::Right),
            _ => false
        };

        self.index += 1;

        if key {
            1
        } else {
            0
        }

        // if address != 0x4016 { return 0 };
        // let mut value = 0;
        //
        // if self.index < 8 && self.buttons[self.index as usize] {
        //     // println!("Controller {} got {}", self.index, self.buttons[self.index as usize]);
        //     value = 1;
        // }
        //
        // self.index += 1;
        //
        // if self.strobe & 1 == 1 {
        //     self.index = 0;
        // }
        //
        // value
    }

    pub fn store(&mut self, address: u16, value: u8) {
        if address != 0x4016 { return };
        self.strobe = value;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }
}
