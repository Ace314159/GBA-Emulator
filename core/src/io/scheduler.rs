use priority_queue::PriorityQueue;

use super::IO;

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
            }
        }
    }
}

pub struct Scheduler {
    pub cycle: usize,
    event_queue: PriorityQueue<EventType, usize>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            cycle: 0,
            event_queue: PriorityQueue::new(),
        }
    }

    pub fn get_next_event(&mut self) -> Option<EventType> {
        if self.event_queue.len() == 0 { return None }
        let (_event_type, cycle) = self.event_queue.peek().unwrap();
        if self.cycle == *cycle {
            Some(self.event_queue.pop().unwrap().0)
        } else { None }
    }

    pub fn add(&mut self, event: Event) {
        self.event_queue.push(event.event_type, event.cycle);
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
}
