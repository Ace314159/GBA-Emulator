mod registers;

use super::{Event, IORegister};
use super::InterruptRequest;

use registers::*;

pub struct Timers {
    pub timers: [Timer; 4],
    pub timers_by_prescaler: [Vec<usize>; 4],
}

impl Timers {
    pub const PRESCALERS: [usize; 4] = [1, 64, 256, 1024];

    pub fn new() -> Timers {
        Timers {
            timers: [
                Timer::new(InterruptRequest::TIMER0_OVERFLOW),
                Timer::new(InterruptRequest::TIMER1_OVERFLOW),
                Timer::new(InterruptRequest::TIMER2_OVERFLOW),
                Timer::new(InterruptRequest::TIMER3_OVERFLOW),
            ],
            timers_by_prescaler: Default::default(),
        }
    }

    pub fn write(&mut self, timer_i: usize, byte: u8, value: u8) -> Option<Event> {
        let timer = &mut self.timers[timer_i];
        let prev_prescaler = if timer.is_count_up() { 4 } else { timer.cnt.prescaler as usize };
        let prev_start = timer.cnt.start;
        let event = timer.write(byte, value);
        let new_start = timer.cnt.start;
        let new_prescaler = if timer.is_count_up() { 4 } else { timer.cnt.prescaler as usize };
        if prev_prescaler != new_prescaler || prev_start != new_start {
            // TODO: Use faster method, but maybe not needed
            if prev_start && prev_prescaler != 4 {
                let pos = self.timers_by_prescaler[prev_prescaler].iter().position(|t| *t == timer_i).unwrap();
                self.timers_by_prescaler[prev_prescaler].remove(pos);
            } else { assert_eq!(self.timers_by_prescaler[prev_prescaler].iter().position(|t| *t == timer_i), None) }
            if new_start && new_prescaler != 4 {
                self.timers_by_prescaler[new_prescaler].push(timer_i);
                self.timers_by_prescaler[new_prescaler].sort();
            }
        }
        event
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    pub reload: u16,
    pub cnt: TMCNT,
    pub started: bool,
    counter: u16,
    pub interrupt: InterruptRequest,
}

impl Timer {
    pub fn new(interrupt: InterruptRequest) -> Timer {
        Timer {
            reload: 0,
            cnt: TMCNT::new(),
            started: false,
            counter: 0,
            interrupt,
        }
    }

    pub fn clock(&mut self) -> (bool, InterruptRequest) {
        if self.cnt.start {
            let (new_counter, overflowed) = self.counter.overflowing_add(1);
            if overflowed {
                self.counter = self.reload;
                let interrupt = if self.cnt.irq { self.interrupt } else { InterruptRequest::empty() };
                return (true, interrupt)
            } else { self.counter = new_counter }
        }
        (false, InterruptRequest::empty())
    }

    pub fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.counter >> 0) as u8,
            1 => (self.counter >> 8) as u8,
            2 | 3 => self.cnt.read(byte - 2),
            _ => unreachable!(),
        }
    }

    pub fn is_count_up(&self) -> bool { self.cnt.count_up }

    pub fn write(&mut self, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => self.reload = self.reload & !0x00FF | (value as u16) << 0,
            1 => self.reload = self.reload & !0xFF00 | (value as u16) << 8,
            2 => {
                let prev_start = self.cnt.start;
                self.cnt.write(0, value);
                if !prev_start && self.cnt.start {
                    // TODO: Add 1 cycle delay
                    self.counter = self.reload;
                }
            },
            3 => { self.cnt.write(1, value); () },
            _ => unreachable!(),
        }
        None
    }
}
