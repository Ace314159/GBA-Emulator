mod registers;

use super::{Event, IORegister};
use super::InterruptRequest;

use registers::*;

pub struct Timers {
    pub timers: [Timer; 4],
}

impl Timers {
    pub fn new() -> Timers {
        Timers {
            timers: [
                Timer::new(InterruptRequest::TIMER0_OVERFLOW),
                Timer::new(InterruptRequest::TIMER1_OVERFLOW),
                Timer::new(InterruptRequest::TIMER2_OVERFLOW),
                Timer::new(InterruptRequest::TIMER3_OVERFLOW),
            ],
        }
    }

    pub fn clock(&mut self) -> (InterruptRequest, [bool; 4]) {
        let mut prev_timer_overflowed = false;
        let mut interrupts = InterruptRequest::empty();
        let mut timers_overflowed = [false; 4];
        for (i, timer) in self.timers.iter_mut().enumerate() {
            let out = timer.clock(prev_timer_overflowed);
            timers_overflowed[i] = out.0;
            prev_timer_overflowed = out.0;
            interrupts |= out.1;
        }
        (interrupts, timers_overflowed)
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    pub reload: u16,
    pub cnt: TMCNT,
    pub started: bool,
    counter: u16,
    prescaler_counter: u16,
    interrupt: InterruptRequest,
}

impl Timer {
    pub fn new(interrupt: InterruptRequest) -> Timer {
        Timer {
            reload: 0,
            cnt: TMCNT::new(),
            started: false,
            counter: 0,
            prescaler_counter: 1,
            interrupt,
        }
    }

    pub fn clock(&mut self, prev_timer_overflowed: bool) -> (bool, InterruptRequest) {
        if self.started {
            let clock = if self.cnt.count_up {
                prev_timer_overflowed
            } else {
                self.prescaler_counter -= 1;
                self.prescaler_counter == 0
            };
            if clock {
                self.prescaler_counter = self.cnt.prescaler_period;
                let (new_counter, overflowed) = self.counter.overflowing_add(1);
                if overflowed {
                    self.counter = self.reload;
                    let interrupt = if self.cnt.irq { self.interrupt } else { InterruptRequest::empty() };
                    return (true, interrupt)
                } else { self.counter = new_counter }
            }
        }
        self.started = self.cnt.start;
        (false, InterruptRequest::empty())
    }
}

impl IORegister for Timer {
    fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => (self.counter >> 0) as u8,
            1 => (self.counter >> 8) as u8,
            2 | 3 => self.cnt.read(byte - 2),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => self.reload = self.reload & !0x00FF | (value as u16) << 0,
            1 => self.reload = self.reload & !0xFF00 | (value as u16) << 8,
            2 => {
                let prev_start = self.cnt.start;
                self.cnt.write(0, value);
                if !prev_start && self.cnt.start {
                    self.counter = self.reload;
                    self.prescaler_counter = self.cnt.prescaler_period;
                }
            },
            3 => { self.cnt.write(1, value); () },
            _ => unreachable!(),
        }
        None
    }
}
