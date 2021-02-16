use std::cell::Cell;

mod pulse;

use pulse::{Counter, Pulse};

pub struct Apu {
    pulse_1: Pulse,
    pulse_2: Pulse,
    noise: Noise,
    triangle: Triangle,

    dmc: Dmc,
    frame_int: Cell<bool>,

    counter: u16,
    mode: Mode,
    int_inhibit: bool,
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    Step4,
    Step5,
}

struct Noise {
    counter: u8,
    enabled: bool,
}

struct Triangle {
    counter: u8,
    enabled: bool,
}

struct Dmc {
    enabled: bool,
    bytes: u8,
    interupt: bool,
}

impl Noise {
    fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
        }
    }

    fn active(&self) -> bool { self.counter > 0 && self.enabled }

    fn halt(&mut self) { self.enabled = false; }

    fn disable(&mut self) {
        self.halt();
        self.counter = 0;
    }

    fn enable(&mut self) { self.enabled = true; }
}

impl Triangle {
    fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
        }
    }

    fn active(&self) -> bool { self.counter > 0 && self.enabled }

    fn halt(&mut self) { self.enabled = false; }

    fn disable(&mut self) {
        self.halt();
        self.counter = 0;
    }

    fn enable(&mut self) { self.enabled = true; }
}

impl Dmc {
    fn new() -> Self {
        Self {
            enabled: false,
            interupt: false,
            bytes: 0,
        }
    }

    fn disable(&mut self) {
        self.enabled = false;
        self.bytes = 0;
    }

    fn enable(&mut self) { self.enabled = true; }
}

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 38, 32, 30,
];

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            noise: Noise::new(),
            triangle: Triangle::new(),
            dmc: Dmc::new(),
            frame_int: Cell::new(false),

            counter: 0,
            mode: Mode::Step4,
            int_inhibit: false,
        }
    }

    pub fn clock(&mut self) {
        self.counter += 1;
        let (_quarter, half) = match (self.counter, self.mode) {
            (3728, _) => (true, false),
            (7456, _) => (true, true),
            (11185, _) => (true, false),
            (14914, Mode::Step4) => {
                if !self.int_inhibit {
                    self.frame_int.set(true);
                }
                (true, true)
            }
            (14915, Mode::Step4) => {
                self.counter = 0;
                (false, false)
            }
            (18640, Mode::Step5) => (true, true),
            (18641, Mode::Step5) => {
                self.counter = 0;
                (false, false)
            }
            (_, _) => (false, false),
        };

        if half {
            self.pulse_1.counter.as_mut().map(Counter::clock);
            self.pulse_2.counter.as_mut().map(Counter::clock);
        }

        self.pulse_1.clock();
        self.pulse_2.clock();
    }

    pub fn get_status(&self) -> u8 {
        self.pulse_1.active() as u8
            | (self.pulse_2.active() as u8) << 1
            | (self.noise.active() as u8) << 2
            | (self.triangle.active() as u8) << 3
            | ((self.dmc.bytes > 0) as u8) << 4
            | (self.frame_int.get() as u8) << 6
            | (self.dmc.interupt as u8) << 7
    }

    pub fn write(&mut self, idx: u16, val: u8) {
        if val == 0 {
            return;
        }
        match idx {
            0x4015 => {
                self.pulse_1.set_enabled(val & 1 != 0);
                self.pulse_2.set_enabled(val & (1 << 1) != 0);

                if val & (1 << 2) != 0 {
                    self.triangle.enable();
                } else {
                    self.triangle.disable();
                }
                if val & (1 << 3) != 0 {
                    self.noise.enable();
                } else {
                    self.noise.disable();
                }
                if val & (1 << 4) != 0 {
                    self.dmc.enable();
                } else {
                    self.dmc.disable();
                }
                self.dmc.interupt = false;
            }
            0x4017 => {
                self.mode = if val & 0x80 != 0 {
                    Mode::Step5
                } else {
                    Mode::Step4
                };
                self.int_inhibit = val & 0x40 != 0;
            }

            0x4000 => self.pulse_1.write_reg_0(val),
            0x4001 => (), // todo!("Pulse Channel 1 Sweep [{:08b}]", val),
            0x4002 => self.pulse_1.write_reg_2(val), // todo!("Pulse Channel 1 Timer Low [{:08b}]", val),
            0x4003 => self.pulse_1.write_reg_3(val),

            0x4004 => (), // todo!("Pulse Channel 2 Control [{:08b}]", val),
            0x4005 => (), // todo!("Pulse Channel 2 Sweep [{:08b}]", val),
            0x4006 => (), // todo!("Pulse Channel 2 Timer Low [{:08b}]", val),
            0x4007 => (), // todo!("Pulse Channel 2 Counter [{:08b}]", val),

            0x4008 => (), // todo!("Triangle Channel Control [{:08b}]", val),
            0x4009 => (), // todo!("Triangle Channel Invalid [{:08b}]", val),
            0x400A => (), // todo!("Triangle Channel Timer Low [{:08b}]", val),
            0x400B => (), // todo!("Triangle Channel Counter [{:08b}]", val),

            0x400C => (), // todo!("Noise Channel Control [{:08b}]", val),
            0x400D => (), // todo!("Noise Channel Invalid [{:08b}]", val),
            0x400E => (), // todo!("Noise Channel Modifier [{:08b}]", val),
            0x400F => (), // todo!("Noise Channel Counter [{:08b}]", val),

            0x4010 => (), // todo!("DMC Channel Control [{:08b}]", val),
            0x4011 => (), // todo!("DMC Channel Sweep [{:08b}]", val),
            0x4012 => (), // todo!("DMC Channel Timer Low [{:08b}]", val),
            0x4013 => (), // todo!("DMC Channel Counter [{:08b}]", val),

            _ => todo!("Unknown address: {:04X}", idx),
        }
    }
}
