use crate::event::{Event, EventType};
use crate::logger::Logger;
use crate::scheduler::EventScheduler;
use crate::task::{Task, TaskBehavior};
use std::rc::{Rc, Weak};
use std::sync::Mutex;

struct MessageTask {
    base: Task,
    shared_queue: Weak<Mutex<Vec<Event>>>,
    logger: Logger,
}

impl MessageTask {
    fn new(name: impl Into<String>, shared_queue: Weak<Mutex<Vec<Event>>>, logger: Logger) -> Self {
        let name = name.into();
        let event_type = EventType::Custom(name.clone());

        Self {
            base: Task::new(name, event_type),
            shared_queue,
            logger,
        }
    }

    fn send_message(&self, recipient: &str, payload: &str) {
        if let Some(queue) = self.shared_queue.upgrade() {
            let event = Event::new(
                EventType::Custom(recipient.to_string()),
                payload.to_string(),
            );
            self.logger.info(format!(
                "{} -> {}: {}",
                self.base.name(),
                recipient,
                event.payload()
            ));
            queue.lock().unwrap().push(event);
        } else {
            self.logger.error(format!(
                "[{}] Failed to send message: message queue is unavailable",
                self.base.name()
            ));
        }
    }
}

impl TaskBehavior for MessageTask {
    fn base(&self) -> &Task {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Task {
        &mut self.base
    }

    fn handle_event(&mut self, event: &Event) {
        self.logger.info(format!(
            "[{}] received: {}",
            self.base.name(),
            event.payload()
        ));

        match (self.base.name(), event.payload()) {
            ("task_a", "start") => self.send_message("task_b", "ping"),
            ("task_b", "ping") => self.send_message("task_a", "pong"),
            ("task_a", "pong") => self.logger.info("[task_a] conversation complete."),
            _ => self.logger.debug(format!(
                "[{}] no follow-up action for payload '{}'.",
                self.base.name(),
                event.payload()
            )),
        }
    }
}

pub fn run_message_passing_example(logger: Logger) {
    logger.info("Message passing example started.");

    let event_queue = Rc::new(Mutex::new(Vec::new()));
    let mut scheduler = EventScheduler::new_with_queue(Rc::clone(&event_queue));

    let task_a = MessageTask::new("task_a", Rc::downgrade(&event_queue), logger.clone());
    let task_b = MessageTask::new("task_b", Rc::downgrade(&event_queue), logger.clone());

    scheduler.register_task(Box::new(task_a));
    scheduler.register_task(Box::new(task_b));
    scheduler.schedule(Event::new(
        EventType::Custom("task_a".into()),
        "start".into(),
    ));

    scheduler.dispatch();
    logger.info("Message passing example finished.");
}
