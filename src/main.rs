mod event;
mod scheduler;
mod task;

use crate::event::{Event, EventType};
use crate::scheduler::EventScheduler;
use crate::task::{Task, TaskBehavior};

struct PrintTask {
    base: Task,
    prefix: String,
}

impl PrintTask {
    fn new(name: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            base: Task::new(name, EventType::Custom("print".into())),
            prefix: prefix.into(),
        }
    }
}

impl TaskBehavior for PrintTask {
    fn base(&self) -> &Task {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Task {
        &mut self.base
    }

    fn handle_event(&mut self, event: &Event) {
        println!("{} [{}] -> {}", self.prefix, self.base.name(), event.payload());
    }
}

fn main() {
    let mut scheduler = EventScheduler::new();

    let print_task = PrintTask::new("printer", "Printer task");
    scheduler.register_task(Box::new(print_task));

    scheduler.schedule(Event::new(EventType::Custom("print".into()), "Hello from the scheduler".into()));
    scheduler.schedule(Event::new(EventType::Custom("print".into()), "Second event".into()));

    scheduler.dispatch();
}
