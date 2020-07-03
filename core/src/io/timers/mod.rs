mod registers;

use super::{Event, EventType, IORegister};
use super::InterruptRequest;

use registers::*;

pub struct Timers {
    pub timers: [Timer; 4],
}

impl Timers {
    pub const PRESCALERS: [usize; 4] = [1, 64, 256, 1024];

    pub fn new() -> Timers {
        Timers {
            timers: [
                Timer::new(0, InterruptRequest::TIMER0_OVERFLOW),
                Timer::new(1, InterruptRequest::TIMER1_OVERFLOW),
                Timer::new(2, InterruptRequest::TIMER2_OVERFLOW),
                Timer::new(3, InterruptRequest::TIMER3_OVERFLOW),
            ],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    pub reload: u16,
    pub cnt: TMCNT,
    pub index: usize,
    pub interrupt: InterruptRequest,
    // Counter Calcuation
    // Count-Up Timing
    counter: u16,
    // Regular Timing
    start_cycle: usize,
    time_till_first_clock: usize,
    timer_len: usize,
}

impl Timer {
    pub fn new(index: usize, interrupt: InterruptRequest) -> Timer {
        Timer {
            reload: 0,
            cnt: TMCNT::new(),
            index,
            interrupt,
            // Counter Calcuation
            // Count-Up Timing
            counter: 0,
            // Regular Timing
            start_cycle: 0,
            time_till_first_clock: 0,
            timer_len: 0,
        }
    }

    pub fn clock(&mut self) -> bool {
        assert!(self.is_count_up());
        if self.cnt.start {
            let (new_counter, overflowed) = self.counter.overflowing_add(1);
            if overflowed {
                self.counter = self.reload;
                return true
            } else { self.counter = new_counter }
        }
        false
    }

    pub fn read(&self, global_cycle: usize, byte: u8) -> u8 {
        let counter = if self.is_count_up() || !self.cnt.start { self.counter } else { self.calc_counter(global_cycle) };
        match byte {
            0 => (counter >> 0) as u8,
            1 => (counter >> 8) as u8,
            2 | 3 => self.cnt.read(byte - 2),
            _ => unreachable!(),
        }
    }

    fn calc_counter(&self, global_cycle: usize) -> u16 {    
        let cycles_passed = global_cycle - self.start_cycle;
        // Counter stores the reload value
        if cycles_passed >= self.time_till_first_clock {
            let cycles_passed = cycles_passed - self.time_till_first_clock;
            self.counter + 1 + cycles_passed as u16 / Timers::PRESCALERS[self.cnt.prescaler as usize] as u16
        } else { self.counter }
    }

    pub fn create_event(&mut self, global_cycle: usize) -> Event {
        self.counter = self.reload;
        self.start_cycle = global_cycle;
        // Syncs prescaler to global cycle
        let prescaler = Timers::PRESCALERS[self.cnt.prescaler as usize];
        // Add 1 for 1 cycle delay in timer start
        self.time_till_first_clock = prescaler - (global_cycle + 1) % prescaler;
        self.timer_len = prescaler * (0x10000 - self.reload as usize - 1);
        Event {
            cycle: global_cycle + self.time_till_first_clock + self.timer_len,
            event_type: EventType::TimerOverflow(self.index),
        }
    }

    pub fn is_count_up(&self) -> bool { self.cnt.count_up }

    pub fn write(&mut self, global_cycle: usize, byte: u8, value: u8) -> Option<Event> {
        match byte {
            0 => self.reload = self.reload & !0x00FF | (value as u16) << 0,
            1 => self.reload = self.reload & !0xFF00 | (value as u16) << 8,
            2 => {
                let prev_start = self.cnt.start;
                self.cnt.write(0, value);
                if !self.is_count_up() {
                    if !prev_start && self.cnt.start {
                        return Some(self.create_event(global_cycle + 1))
                    } else if prev_start && !self.cnt.start {
                        self.counter = self.calc_counter(global_cycle);
                        // Removes event
                        return Some(Event {
                            cycle: global_cycle - 1,
                            event_type: EventType::TimerOverflow(self.index),
                        })
                    } else if self.cnt.start {
                        // TODO: Prescaler Value changed
                        todo!();
                    }
                } else {
                    if !prev_start && self.cnt.start {
                        println!("Starting Count-Up Timer");
                        self.counter = self.reload;
                    }
                }
            },
            3 => { self.cnt.write(1, value); () },
            _ => unreachable!(),
        }
        None
    }
}
