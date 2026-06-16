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
    timestamp: u64,
}

impl Event {
    pub fn new(event_type: EventType, payload: String) -> Self {
        Self {
            event_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

pub struct EventHandler {
    handlers: Vec<Box<dyn Fn(&Event) + Send>>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(&Event) + Send + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub fn emit(&self, event: &Event) {
        for handler in &self.handlers {
            handler(event);
        }
    }
}
