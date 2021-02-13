use std::cell::Cell;

pub struct Apu {
    pulse_1: Channel,
    pulse_2: Channel,
    noise: Channel,
    triangle: Channel,

    dmc: Dmc,
    frame_int: Cell<bool>,
}

struct Channel {
    counter: u8,
    enabled: bool,
}

struct Dmc {
    enabled: bool,
    bytes: u8,
    interupt: bool,
}

impl Channel {
    fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
        }
    }

    fn active(&self) -> bool {
        self.counter > 0 && self.enabled
    }

    fn halt(&mut self) {
        self.enabled = false;
    }

    fn disable(&mut self) {
        self.halt();
        self.counter = 0;
    }

    fn enable(&mut self) {
        self.enabled = true;
    }
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

    fn enable(&mut self) {
        self.enabled = true;
    }
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse_1: Channel::new(),
            pulse_2: Channel::new(),
            noise: Channel::new(),
            triangle: Channel::new(),
            dmc: Dmc::new(),
            frame_int: Cell::new(false),
        }
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
                if val & 1 != 0 {
                    self.pulse_1.enable();
                } else {
                    self.pulse_1.disable();
                }
                if val & (1 << 1) != 0 {
                    self.pulse_2.enable();
                } else {
                    self.pulse_2.disable();
                }
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
            0x4017 => (), //todo!("Frame counter [{:08b}]", val),
            _ => {
                let reg = match idx % 4 {
                    0 => "Control",
                    1 => "Sweep",
                    2 => "Timer Low",
                    3 => "Counter",
                    _ => unreachable!(),
                };
                match idx & 0xFC {
                    0x4000 => todo!("Pulse Channel 1 {} [{:08b}]", reg, val),
                    0x4004 => todo!("Pulse Channel 2 {} [{:08b}]", reg, val),
                    0x400C => todo!("Noise Channel {} [{:08b}]", reg, val),
                    0x4008 => todo!("Triangle Channel {} [{:08b}]", reg, val),
                    _ => (),
                }
            }
        }
    }
}