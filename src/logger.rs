use crate::event::{Event, EventType};
use crate::task::{Task, TaskBehavior, TaskShutdown, TaskStartup, TaskState};
use std::cell::RefCell;
use std::rc::Rc;

/// Basic log levels, ordered from most to least verbose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Shared logger handle. Cloning shares the same underlying logger state.
#[derive(Clone)]
pub struct Logger {
    inner: Rc<RefCell<LoggerState>>,
}

struct LoggerState {
    min_level: LogLevel,
}

impl Logger {
    /// Creates a logger that emits records at or above `min_level`.
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            inner: Rc::new(RefCell::new(LoggerState { min_level })),
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        self.inner.borrow_mut().min_level = level;
    }

    pub fn level(&self) -> LogLevel {
        self.inner.borrow().min_level
    }

    /// Logs a message at the given level. Errors and warnings are written to
    /// stderr; everything else goes to stdout.
    pub fn log(&self, level: LogLevel, message: impl AsRef<str>) {
        if level < self.inner.borrow().min_level {
            return;
        }

        let line = format!("[{}] {}", level.as_str(), message.as_ref());
        match level {
            LogLevel::Warn | LogLevel::Error => eprintln!("{}", line),
            _ => println!("{}", line),
        }
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Error, message);
    }
}

/// A persistent task, derived from the `Task` base type, that consumes log
/// events from the scheduler and forwards them to a shared [`Logger`].
///
/// Log events use [`EventType::Custom`] with the value `"log"` and a payload of
/// the form `"<LEVEL>|<message>"`.
pub struct LoggerTask {
    base: Task,
    logger: Logger,
}

impl LoggerTask {
    pub fn new(name: impl Into<String>, logger: Logger) -> Self {
        Self {
            base: Task::new(name, EventType::Custom("log".into())),
            logger,
        }
    }

    /// Returns a clone of the shared logger handle used by this task.
    pub fn logger(&self) -> Logger {
        self.logger.clone()
    }

    /// Builds a log event that the scheduler can route to this task.
    pub fn log_event(level: LogLevel, message: impl AsRef<str>) -> Event {
        Event::new(
            EventType::Custom("log".into()),
            format!("{}|{}", level.as_str(), message.as_ref()),
        )
    }

    fn parse_payload(payload: &str) -> (LogLevel, &str) {
        match payload.split_once('|') {
            Some((level, message)) => (Self::parse_level(level), message),
            None => (LogLevel::Info, payload),
        }
    }

    fn parse_level(level: &str) -> LogLevel {
        match level {
            "DEBUG" => LogLevel::Debug,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

impl TaskBehavior for LoggerTask {
    fn base(&self) -> &Task {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Task {
        &mut self.base
    }

    fn handle_event(&mut self, event: &Event) {
        let (level, message) = Self::parse_payload(event.payload());
        self.logger.log(level, message);
    }
}

impl TaskStartup for LoggerTask {
    fn on_startup(&mut self) -> Result<(), String> {
        self.base_mut().set_state(TaskState::Running);
        self.logger
            .debug(format!("logger task '{}' started", self.base.name()));
        Ok(())
    }
}

impl TaskShutdown for LoggerTask {
    fn on_shutdown(&mut self) -> Result<(), String> {
        self.logger
            .debug(format!("logger task '{}' stopped", self.base.name()));
        self.base_mut().set_state(TaskState::Stopped);
        Ok(())
    }
}
