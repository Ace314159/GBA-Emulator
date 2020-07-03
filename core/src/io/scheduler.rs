use std::cmp::{Eq, PartialEq, Ord, PartialOrd, Ordering};

use super::IO;

impl IO {
    pub fn handle_events(&mut self) {
        self.scheduler.cycle += 1;
        for event in self.scheduler.get_next_events().iter() {
            self.handle_event(*event);
        }
    }

    pub fn handle_event(&mut self, event: EventType) {
        match event {
            /*EventType::TimerPrescaler(prescaler) => {
                assert_eq!(self.cycle % Timers::PRESCALERS[prescaler], 0);
                for timer in self.timers.timers_by_prescaler[prescaler].clone().iter() {
                    assert!(!self.timers.timers[*timer].is_count_up());
                    let (overflowed, interrupt_request) = self.timers.timers[*timer].clock();
                    if overflowed { self.handle_event(EventType::TimerOverflow(*timer)) }
                    self.interrupt_controller.request |= interrupt_request;
                }
                self.event_queue.push(Event {
                    cycle: self.cycle + Timers::PRESCALERS[prescaler],
                    event_type: EventType::TimerPrescaler(prescaler),
                });
            },*/
            EventType::TimerOverflow(timer) => {
                if self.timers.timers[timer].cnt.irq {
                    self.interrupt_controller.request |= self.timers.timers[timer].interrupt
                }
                // Cascade Timers
                if timer + 1 < self.timers.timers.len() && self.timers.timers[timer + 1].is_count_up() {
                    if self.timers.timers[timer + 1].clock() { self.handle_event(EventType::TimerOverflow(timer + 1)) }
                }
                if !self.timers.timers[timer].is_count_up() {
                    self.scheduler.add(self.timers.timers[timer].create_event(self.scheduler.cycle));
                }
                // Sound FIFOs
                self.apu.on_timer_overflowed(timer);
            }
        }
    }
}

pub struct Scheduler {
    pub cycle: usize,
    event_queue: Vec<Event>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            cycle: 0,
            event_queue: Vec::new(),
        }
    }

    pub fn get_next_events(&mut self) -> Vec<EventType> {
        if self.event_queue.len() > 0 {
            let mut i = self.event_queue.len() - 1;
            let mut events = Vec::new();
            while self.event_queue[i].cycle == self.cycle {
                let event = self.event_queue.swap_remove(i);
                events.push(event.event_type);
                if i == 0 { break }
                i -= 1;
            }
            events
        } else { Vec::new() }
    }

    pub fn add(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub fn remove(&mut self, event_type: EventType) {
        if let Some(pos) = self.event_queue.iter().position(|e| e.event_type == event_type) {
            self.event_queue.swap_remove(pos);
        }
    }

    pub fn sort(&mut self) {
        self.event_queue.sort();
        self.event_queue.reverse();
    }
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct Event {
    pub cycle: usize,
    pub event_type: EventType,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed since BinaryHeap is a max heap
        self.cycle.cmp(&other.cycle)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reversed since BinaryHeap is a max heap
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cycle == other.cycle
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    TimerOverflow(usize),
}
