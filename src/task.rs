use crate::event::{Event, EventType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Initialized,
    Running,
    Paused,
    Stopped,
    Failed,
}

#[derive(Debug)]
pub struct Task {
    name: String,
    event_type: EventType,
    state: TaskState,
    retry_count: u32,
    max_retries: u32,
    properties: HashMap<String, String>,
}

impl Task {
    pub fn new(name: impl Into<String>, event_type: EventType) -> Self {
        Self {
            name: name.into(),
            event_type,
            state: TaskState::Initialized,
            retry_count: 0,
            max_retries: 3,
            properties: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    pub fn reset_retry(&mut self) {
        self.retry_count = 0;
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    pub fn set_max_retries(&mut self, max: u32) {
        self.max_retries = max;
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }
}

pub trait TaskStartup {
    fn on_startup(&mut self) -> Result<(), String>;
}

pub trait TaskShutdown {
    fn on_shutdown(&mut self) -> Result<(), String>;
}

pub trait TaskRetry {
    fn on_retry(&mut self) -> Result<(), String>;
    fn should_retry(&self) -> bool;
}

pub trait TaskPausable {
    fn on_pause(&mut self) -> Result<(), String>;
    fn on_resume(&mut self) -> Result<(), String>;
    fn is_paused(&self) -> bool;
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

pub struct TaskFactory;

impl TaskFactory {
    pub fn create_task(name: impl Into<String>, event_type: EventType) -> Task {
        Task::new(name, event_type)
    }

    pub fn create_task_with_retries(
        name: impl Into<String>,
        event_type: EventType,
        max_retries: u32,
    ) -> Task {
        let mut task = Task::new(name, event_type);
        task.set_max_retries(max_retries);
        task
    }

    pub fn create_task_with_properties(
        name: impl Into<String>,
        event_type: EventType,
        properties: HashMap<String, String>,
    ) -> Task {
        let mut task = Task::new(name, event_type);
        for (key, value) in properties {
            task.set_property(key, value);
        }
        task
    }
}
