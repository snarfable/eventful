#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Scheduled,
    Manual,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Event {
    event_type: EventType,
    payload: String,
}

impl Event {
    pub fn new(event_type: EventType, payload: String) -> Self {
        Self { event_type, payload }
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}
