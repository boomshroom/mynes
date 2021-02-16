use std::num::Wrapping;

use super::LENGTH_TABLE;

pub struct Pulse {
    enabled: bool,
    sample: f64,

    sequence: Sequencer,
    sweep: Sweep,
    pub counter: Option<Counter>,
}

pub struct Sweep {
    enabled: bool,
    period: u8,
    dir: Direction,
    shift: u8,
}

pub struct Sequencer {
    sequence: u32,
    timer: Wrapping<u16>,
    reload: Wrapping<u16>,
    output: u8,
}

pub struct Counter {
    running: bool,
    count: u8,
}

pub enum Direction {
    Lower,
    Higher,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            enabled: false,
            sample: 0.0,

            sequence: Sequencer::new(),
            sweep: Sweep::new(),
            counter: None,
        }
    }

    pub fn clock(&mut self) {
        if self.enabled {
            self.sequence.clock(|s| s.rotate_right(1));
        }

        self.sample = self.sequence.output as f64;
    }

    pub fn active(&self) -> bool {
        self.counter.as_ref().map_or(false, |c| c.count > 0) && self.enabled
    }

    pub fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
        match (enable, self.counter.as_ref()) {
            (true, None) => self.counter = Some(Counter::new()),
            (false, _) => self.counter = None,
            _ => (),
        };
    }

    pub fn write_reg_0(&mut self, val: u8) {
        match (val >> 6) % 4 {
            0 => self.sequence.sequence = 0b00000001,
            1 => self.sequence.sequence = 0b00000011,
            2 => self.sequence.sequence = 0b00001111,
            3 => self.sequence.sequence = 0b11111100,
            _ => unreachable!(),
        }
        if let Some(ref mut c) = self.counter {
            c.running = val & 0x20 == 0;
        }
    }

    pub fn write_reg_2(&mut self, val: u8) {
        self.sequence.reload.0 = (self.sequence.reload.0 & 0xFF00) | val as u16;
    }

    pub fn write_reg_3(&mut self, val: u8) {
        self.sequence.reload.0 = (self.sequence.reload.0 & 0x00FF) | ((val as u16 & 0x07) << 8);
        self.sequence.reset();

        if let Some(ref mut c) = self.counter {
            c.count = LENGTH_TABLE[usize::from(val >> 3)];
        }
    }
}

impl Sweep {
    pub fn new() -> Self {
        Sweep {
            enabled: false,
            period: 0,
            dir: Direction::Lower,
            shift: 0,
        }
    }

    pub fn set_enabled(&mut self, enable: bool) { self.enabled = enable; }
}

impl Sequencer {
    fn new() -> Self {
        Sequencer {
            sequence: 0,
            timer: Wrapping(0),
            reload: Wrapping(0),
            output: 0,
        }
    }

    fn reset(&mut self) { self.timer = self.reload; }

    pub fn clock(&mut self, f: impl FnOnce(u32) -> u32) -> u8 {
        self.timer -= Wrapping(1);
        if self.timer.0 == 0xFFFF {
            self.timer = self.reload + Wrapping(1);
            self.sequence = f(self.sequence);
            self.output = (self.sequence & 1) as u8;
        }
        self.output
    }
}

impl Counter {
    fn new() -> Self {
        Self {
            running: false,
            count: 0,
        }
    }
    pub fn clock(&mut self) -> bool {
        if self.running && self.count > 0 {
            self.count -= 1;
        }
        self.count > 0
    }
}
