use crate::event::Event;
use crate::task::TaskBehavior;
use std::rc::Rc;
use std::sync::Mutex;

pub struct EventScheduler {
    tasks: Vec<Box<dyn TaskBehavior>>,
    event_queue: Rc<Mutex<Vec<Event>>>,
    running: bool,
}

impl EventScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            event_queue: Rc::new(Mutex::new(Vec::new())),
            running: false,
        }
    }

    pub fn new_with_queue(event_queue: Rc<Mutex<Vec<Event>>>) -> Self {
        Self {
            tasks: Vec::new(),
            event_queue,
            running: false,
        }
    }

    pub fn register_task(&mut self, task: Box<dyn TaskBehavior>) {
        self.tasks.push(task);
    }

    pub fn unregister_task(&mut self, task_name: &str) {
        self.tasks.retain(|task| task.name() != task_name);
    }

    pub fn schedule(&self, event: Event) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.push(event);
    }

    pub fn dispatch(&mut self) {
        self.running = true;
        loop {
            let event = {
                let mut queue = self.event_queue.lock().unwrap();
                queue.pop()
            };

            match event {
                Some(event) => {
                    for task in self.tasks.iter_mut() {
                        if task.matches(&event) {
                            task.handle_event(&event);
                        }
                    }
                }
                None => break,
            }
        }
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn pending_events(&self) -> usize {
        self.event_queue.lock().unwrap().len()
    }

    pub fn registered_tasks(&self) -> usize {
        self.tasks.len()
    }

    pub fn clear_queue(&mut self) {
        self.event_queue.lock().unwrap().clear();
    }
}
