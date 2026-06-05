use crate::event::Event;
use crate::task::TaskBehavior;

pub struct EventScheduler {
    tasks: Vec<Box<dyn TaskBehavior>>,
    event_queue: Vec<Event>,
}

impl EventScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn register_task(&mut self, task: Box<dyn TaskBehavior>) {
        self.tasks.push(task);
    }

    pub fn schedule(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub fn dispatch(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            for task in self.tasks.iter_mut() {
                if task.matches(&event) {
                    task.handle_event(&event);
                }
            }
        }
    }
}
