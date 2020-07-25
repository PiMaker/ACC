use serde::Serialize;

use super::consts::*;

const DUR_PREAMBLE: u32 = 4400;
const DUR_ON: u32 = 1582;
const DUR_OFF: u32 = 503;
const DUR_WAIT: u32 = 576;

#[derive(Clone)]
pub enum SignalType {
    Invalid,
    Preamble,
    One,
    Zero,
}

#[derive(Clone)]
pub struct Signal {
    pub signal_type: SignalType,
    pub duration: u64,
}

impl Signal {
    pub fn one() -> Self {
        Signal {
            signal_type: SignalType::One,
            duration: DUR_ON as u64 * 1000,
        }
    }

    pub fn zero() -> Self {
        Signal {
            signal_type: SignalType::Zero,
            duration: DUR_OFF as u64 * 1000,
        }
    }

    pub fn decode(duration: u64) -> Self {
        if duration > 350000 && duration < 650000 {
            return Signal { signal_type: SignalType::Zero, duration };
        } else if duration > 1350000 && duration < 1650000 {
            return Signal { signal_type: SignalType::One, duration };
        } else if duration > 4000000 && duration < 6000000 {
            return Signal { signal_type: SignalType::Preamble, duration };
        } else {
            return Signal { signal_type: SignalType::Invalid, duration };
        }
    }

    fn bit(&self) -> u8 {
        match self.signal_type {
            SignalType::Zero => 0,
            SignalType::One => 1,
            _ => panic!("Cannot get 'bit' value of non-value signal")
        }
    }
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.signal_type {
            SignalType::Zero => {
                write!(f, "0")
            },
            SignalType::One => {
                write!(f, "1")
            },
            _ => Ok(())
        }
    }
}

pub fn verify_and_trim(signals: &[Signal]) -> Option<Vec<&Signal>> {
    let signals = signals.iter().filter(|s| {
        match s.signal_type {
            SignalType::Zero | SignalType::One => true,
            SignalType::Preamble | SignalType::Invalid => false
        }
    }).collect::<Vec<&Signal>>();

    if signals.len() != 48*2 {
        return None;
    }

    let first = signals.iter().take(48);
    let second = signals.iter().skip(48);
    let equal = first.zip(second).all(|(a, b)|
        std::mem::discriminant(&a.signal_type) == std::mem::discriminant(&b.signal_type));

    if !equal {
        return None;
    }

    Some(signals[..48].to_vec())
}

fn signal_to_u8(signals: &[&Signal]) -> u8 {
    assert_eq!(signals.len(), 8);
    // MSB comes first in transmission order
    signals[7].bit() |
        (signals[6].bit() << 1) |
        (signals[5].bit() << 2) |
        (signals[4].bit() << 3) |
        (signals[3].bit() << 4) |
        (signals[2].bit() << 5) |
        (signals[1].bit() << 6) |
        (signals[0].bit() << 7)
}

fn signal_from_u8(data: u8) -> Vec<Signal> {
    vec!(
        if data & 0b10000000 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b1000000 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b100000 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b10000 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b1000 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b100 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b10 != 0 { Signal::one() } else { Signal::zero() },
        if data & 0b1 != 0 { Signal::one() } else { Signal::zero() },
    )
}

pub fn verify_checksum(signals: &[&Signal]) -> bool {
    // 6 bytes per transmission, every second byte is inverse of correponding
    let bytes: Vec<u8> = signals.chunks(8).map(signal_to_u8).collect();
    for i in 0..bytes.len()/2 {
        if bytes[i*2] != !bytes[i*2+1] {
            return false
        }
    }
    return true;
}

pub struct AcCtrl {
    flags: u8,
    fan_and_state: u8,
    temp_and_mode: u8,
}

impl AcCtrl {
    pub fn from_signals(signals: &[&Signal]) -> AcCtrl {
        let signals = signals.chunks(8).map(signal_to_u8).collect::<Vec<u8>>();
        let mut iter = if signals.len() == 6 {
            signals.iter().step_by(2)
        } else if signals.len() == 3 {
            signals.iter().step_by(1)
        } else {
            panic!("cannot convert signals with length != (3 || 6) to AcCtrl");
        };

        AcCtrl {
            flags: *iter.next().unwrap(),
            fan_and_state: *iter.next().unwrap(),
            temp_and_mode: *iter.next().unwrap(),
        }
    }

    pub fn to_signals(&self) -> Vec<Signal> {
        let mut nested = vec!(
            signal_from_u8(self.flags),
            signal_from_u8(!self.flags),
            signal_from_u8(self.fan_and_state),
            signal_from_u8(!self.fan_and_state),
            signal_from_u8(self.temp_and_mode),
            signal_from_u8(!self.temp_and_mode),
        );
        nested.drain(..).flatten().collect()
    }

    pub fn new(temp: u8, fan: FanSpeed, mode: Mode, on: bool) -> Self {
        AcCtrl {
            flags: FLAGS_DEF,
            fan_and_state: if on { fan as i32 as u8 } else { FanSpeed::Off as i32 as u8 } << 4
                | if on { STATE_ON } else { STATE_OFF },
            temp_and_mode: mode.clone() as i32 as u8 | if on {
                if std::mem::discriminant(&mode) == std::mem::discriminant(&Mode::Fan) {
                    MAP_TEMP.get(&0).unwrap()
                } else {
                    MAP_TEMP.get(&temp).unwrap()
                }
            } else {
                MAP_TEMP.get(&0).unwrap()
            } << 4,
        }
    }
}

#[derive(Serialize)]
struct SignalJson<'a> {
    generated: &'a[u32],
}

pub fn signals_to_json(signals: &[Signal]) -> String {
    let mut durs = Vec::new();

    for i in 0..=1 {
        if i > 0 {
            durs.push(5000);
        }

        durs.push(DUR_PREAMBLE);
        durs.push(DUR_PREAMBLE);
        durs.push(DUR_WAIT);

        for s in signals {
            match s.signal_type {
                SignalType::Zero => {
                    durs.push(DUR_OFF);
                },
                SignalType::One =>  {
                    durs.push(DUR_ON);
                },
                _ => ()
            }
            durs.push(DUR_WAIT);
        }
    }

    let sj = SignalJson {
        generated: &durs,
    };

    return serde_json::json!(sj).to_string();
}
