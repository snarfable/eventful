mod event;
mod examples;
mod logger;
mod scheduler;
mod task;

use crate::event::{Event, EventType};
use crate::examples::message_passing;
use crate::logger::{LogLevel, Logger, LoggerTask};
use crate::scheduler::EventScheduler;
use crate::task::{
    Task, TaskBehavior, TaskFactory, TaskPausable, TaskShutdown, TaskStartup, TaskState,
};
use std::collections::HashMap;

struct PrintTask {
    base: Task,
    prefix: String,
    logger: Logger,
}

impl PrintTask {
    fn new(name: impl Into<String>, prefix: impl Into<String>, logger: Logger) -> Self {
        Self {
            base: Task::new(name, EventType::Custom("print".into())),
            prefix: prefix.into(),
            logger,
        }
    }

    fn from_factory(prefix: impl Into<String>, logger: Logger) -> Self {
        let task = TaskFactory::create_task("printer", EventType::Custom("print".into()));
        Self {
            base: task,
            prefix: prefix.into(),
            logger,
        }
    }

    fn with_properties(
        name: impl Into<String>,
        prefix: impl Into<String>,
        properties: HashMap<String, String>,
        logger: Logger,
    ) -> Self {
        let base = TaskFactory::create_task_with_properties(
            name,
            EventType::Custom("print".into()),
            properties,
        );
        Self {
            base,
            prefix: prefix.into(),
            logger,
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
        self.logger.info(format!(
            "{} [{}] -> {} (state: {:?})",
            self.prefix,
            self.base.name(),
            event.payload(),
            self.base.state()
        ));
    }
}

impl TaskStartup for PrintTask {
    fn on_startup(&mut self) -> Result<(), String> {
        self.logger
            .info(format!("Starting task: {}", self.base.name()));
        self.base_mut().set_state(TaskState::Running);
        Ok(())
    }
}

impl TaskShutdown for PrintTask {
    fn on_shutdown(&mut self) -> Result<(), String> {
        self.logger
            .info(format!("Shutting down task: {}", self.base.name()));
        self.base_mut().set_state(TaskState::Stopped);
        Ok(())
    }
}

impl TaskPausable for PrintTask {
    fn on_pause(&mut self) -> Result<(), String> {
        self.logger
            .info(format!("Pausing task: {}", self.base.name()));
        self.base_mut().set_state(TaskState::Paused);
        Ok(())
    }

    fn on_resume(&mut self) -> Result<(), String> {
        self.logger
            .info(format!("Resuming task: {}", self.base.name()));
        self.base_mut().set_state(TaskState::Running);
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.base().state() == TaskState::Paused
    }
}

fn main() {
    let logger = Logger::new(LogLevel::Debug);
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("message-passing") => message_passing::run_message_passing_example(logger),
        Some(other) => {
            logger.warn(format!(
                "Unknown example '{}'; running default scheduler demo.",
                other
            ));
            run_default_scheduler(logger);
        }
        None => run_default_scheduler(logger),
    }
}

fn run_default_scheduler(logger: Logger) {
    let mut scheduler = EventScheduler::new();

    // Register the persistent logger task so log events routed through the
    // scheduler are emitted via the shared Logger.
    let logger_task = LoggerTask::new("logger", logger.clone());
    scheduler.register_task(Box::new(logger_task));

    // Create task using TaskFactory
    let print_task = PrintTask::from_factory("Printer task", logger.clone());
    scheduler.register_task(Box::new(print_task));

    // Create task with custom properties
    let mut properties = HashMap::new();
    properties.insert("level".to_string(), "info".to_string());
    properties.insert("tag".to_string(), "system".to_string());

    let print_task_with_props =
        PrintTask::with_properties("system_logger", "System Logger", properties, logger.clone());
    scheduler.register_task(Box::new(print_task_with_props));

    logger.info(format!(
        "Scheduler initialized with {} registered tasks",
        scheduler.registered_tasks()
    ));

    scheduler.schedule(Event::new(
        EventType::Custom("print".into()),
        "Hello from the scheduler".into(),
    ));
    scheduler.schedule(Event::new(
        EventType::Custom("print".into()),
        "Second event".into(),
    ));

    logger.info(format!(
        "Queued {} events for dispatch",
        scheduler.pending_events()
    ));

    scheduler.dispatch();

    logger.info(format!(
        "Dispatch complete. Scheduler running: {}",
        scheduler.is_running()
    ));
}
