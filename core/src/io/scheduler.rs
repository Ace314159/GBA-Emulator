use std::cmp::Reverse;

use priority_queue::PriorityQueue;

use super::IO;
use crate::gba;

impl IO {
    pub fn handle_events(&mut self) {
        self.scheduler.cycle += 1;
        while let Some(event) = self.scheduler.get_next_event() {
            self.handle_event(event);
        }
    }

    pub fn handle_event(&mut self, event: EventType) {
        match event {
            EventType::TimerOverflow(timer) => {
                if self.timers.timers[timer].cnt.irq {
                    self.interrupt_controller.request |= self.timers.timers[timer].interrupt
                }
                // Cascade Timers
                if timer + 1 < self.timers.timers.len() && self.timers.timers[timer + 1].is_count_up() {
                    if self.timers.timers[timer + 1].clock() { self.handle_event(EventType::TimerOverflow(timer + 1)) }
                }
                if !self.timers.timers[timer].is_count_up() {
                    self.timers.timers[timer].reload();
                    self.timers.timers[timer].create_event(&mut self.scheduler, 0);
                }
                // Sound FIFOs
                self.apu.on_timer_overflowed(timer);
            },
            EventType::FrameSequencer(step) => {
                self.apu.clock_sequencer(step);
                self.scheduler.add(Event {
                    cycle: self.scheduler.cycle + (gba::CLOCK_FREQ / 512),
                    event_type: EventType::FrameSequencer((step + 1) % 8),
                });
            },
        }
    }
}

pub struct Scheduler {
    pub cycle: usize,
    event_queue: PriorityQueue<EventType, Reverse<usize>>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let mut queue = PriorityQueue::new();
        queue.push(EventType::FrameSequencer(0), Reverse(gba::CLOCK_FREQ / 512));
        Scheduler {
            cycle: 0,
            event_queue: queue,
        }
    }

    pub fn get_next_event(&mut self) -> Option<EventType> {
        // There should always be at least one event
        let (_event_type, cycle) = self.event_queue.peek().unwrap();
        if Reverse(self.cycle) == *cycle {
            Some(self.event_queue.pop().unwrap().0)
        } else { None }
    }

    pub fn add(&mut self, event: Event) {
        self.event_queue.push(event.event_type, Reverse(event.cycle));
    }

    pub fn remove(&mut self, event_type: EventType) {
        self.event_queue.remove(&event_type);
    }
}

pub struct Event {
    pub cycle: usize,
    pub event_type: EventType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    TimerOverflow(usize),
    FrameSequencer(usize),
}
