use std::collections::HashMap;
use lazy_static::lazy_static;

pub enum FanSpeed {
    Auto = 0b1011,
    Low = 0b1001,
    Medium = 0b0101,
    High = 0b0011,
    Off = 0b0111,
}

#[derive(Clone)]
pub enum Mode {
    Cool = 0b0000,
    Heat = 0b1100,
    Dry = 0b0100,
    Auto = 0b1000,
    Fan = -1, // Dry with temp=off
}

lazy_static! {
    pub static ref MAP_TEMP: HashMap<u8, u8> = {
        let mut m = HashMap::new();
        m.insert(17, 0b0000);
        m.insert(18, 0b0001);
        m.insert(19, 0b0011);
        m.insert(20, 0b0010);
        m.insert(21, 0b0110);
        m.insert(22, 0b0111);
        m.insert(23, 0b0101);
        m.insert(24, 0b0100);
        m.insert(25, 0b1100);
        m.insert(26, 0b1101);
        m.insert(27, 0b1001);
        m.insert(28, 0b1000);
        m.insert(29, 0b1010);
        m.insert(30, 0b1011);
        m.insert(0, 0b1110); // off
        m
    };
}

pub const FLAGS_DEF: u8 = 0b10110010;
pub const FLAGS_TOGGLE: u8 = 0b10110101;

pub const STATE_ON: u8 = 0b1111;
pub const STATE_OFF: u8 = 0b1011;
pub const STATE_TOGGLE: u8 = 0b0101;
