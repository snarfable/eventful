use crate::event::{Event, EventType};

#[derive(Debug)]
pub struct Task {
    name: String,
    event_type: EventType,
}

impl Task {
    pub fn new(name: impl Into<String>, event_type: EventType) -> Self {
        Self {
            name: name.into(),
            event_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }
}

pub trait TaskBehavior {
    fn base(&self) -> &Task;

    fn base_mut(&mut self) -> &mut Task;

    fn handle_event(&mut self, event: &Event);

    fn matches(&self, event: &Event) -> bool {
        self.base().event_type() == event.event_type()
    }

    fn name(&self) -> &str {
        self.base().name()
    }
}
