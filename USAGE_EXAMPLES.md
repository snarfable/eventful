# Task Scheduler - Usage Examples

This document provides practical examples for using the enhanced task scheduler system.

---

## Example 1: Basic Task with Lifecycle

Create a task that implements startup and shutdown hooks:

```rust
use crate::task::{Task, TaskBehavior, TaskStartup, TaskShutdown, TaskState};
use crate::event::{Event, EventType};

struct DatabaseTask {
    base: Task,
    connection: Option<String>,
}

impl DatabaseTask {
    fn new(name: &str) -> Self {
        Self {
            base: Task::new(name, EventType::Custom("db".into())),
            connection: None,
        }
    }
}

impl TaskBehavior for DatabaseTask {
    fn base(&self) -> &Task { &self.base }
    fn base_mut(&mut self) -> &mut Task { &mut self.base }
    
    fn handle_event(&mut self, event: &Event) {
        if let Some(ref conn) = self.connection {
            println!("[{}] Query: {} -> Connection: {}", 
                self.base.name(), event.payload(), conn);
        }
    }
}

impl TaskStartup for DatabaseTask {
    fn on_startup(&mut self) -> Result<(), String> {
        self.connection = Some("Database connected".to_string());
        self.base_mut().set_state(TaskState::Running);
        println!("[{}] Database connection established", self.base.name());
        Ok(())
    }
}

impl TaskShutdown for DatabaseTask {
    fn on_shutdown(&mut self) -> Result<(), String> {
        self.connection = None;
        self.base_mut().set_state(TaskState::Stopped);
        println!("[{}] Database connection closed", self.base.name());
        Ok(())
    }
}
```

---

## Example 2: Pausable Task

Create a task that can be paused and resumed:

```rust
use crate::task::{Task, TaskBehavior, TaskPausable, TaskState};
use crate::event::{Event, EventType};

struct ProcessingTask {
    base: Task,
    items_processed: u32,
}

impl ProcessingTask {
    fn new(name: &str) -> Self {
        Self {
            base: Task::new(name, EventType::Custom("process".into())),
            items_processed: 0,
        }
    }
}

impl TaskBehavior for ProcessingTask {
    fn base(&self) -> &Task { &self.base }
    fn base_mut(&mut self) -> &mut Task { &mut self.base }
    
    fn handle_event(&mut self, event: &Event) {
        if !self.is_paused() {
            self.items_processed += 1;
            println!("[{}] Processing: {} (total: {})", 
                self.base.name(), event.payload(), self.items_processed);
        } else {
            println!("[{}] Task is paused, skipping: {}", 
                self.base.name(), event.payload());
        }
    }
}

impl TaskPausable for ProcessingTask {
    fn on_pause(&mut self) -> Result<(), String> {
        self.base_mut().set_state(TaskState::Paused);
        println!("[{}] Paused after {} items", 
            self.base.name(), self.items_processed);
        Ok(())
    }

    fn on_resume(&mut self) -> Result<(), String> {
        self.base_mut().set_state(TaskState::Running);
        println!("[{}] Resumed processing", self.base.name());
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.base().state() == TaskState::Paused
    }
}
```

---

## Example 3: Task with Configuration Properties

Create a task configured via properties:

```rust
use crate::task::{Task, TaskBehavior, TaskFactory};
use crate::event::{Event, EventType};
use std::collections::HashMap;

struct ConfigurableTask {
    base: Task,
}

impl ConfigurableTask {
    fn with_config(
        name: &str,
        log_level: &str,
        retry_policy: &str,
    ) -> Self {
        let mut props = HashMap::new();
        props.insert("log_level".to_string(), log_level.to_string());
        props.insert("retry_policy".to_string(), retry_policy.to_string());
        
        Self {
            base: TaskFactory::create_task_with_properties(
                name,
                EventType::Custom("config".into()),
                props,
            ),
        }
    }
}

impl TaskBehavior for ConfigurableTask {
    fn base(&self) -> &Task { &self.base }
    fn base_mut(&mut self) -> &mut Task { &mut self.base }
    
    fn handle_event(&mut self, event: &Event) {
        let log_level = self.base.get_property("log_level")
            .unwrap_or("INFO");
        let retry_policy = self.base.get_property("retry_policy")
            .unwrap_or("exponential");
        
        println!("[{}] [{}] {} (retry: {})", 
            self.base.name(), log_level, event.payload(), retry_policy);
    }
}

// Usage:
let task = ConfigurableTask::with_config("api_client", "DEBUG", "exponential");
```

---

## Example 4: Retryable Task

Create a task with built-in retry logic:

```rust
use crate::task::{Task, TaskBehavior, TaskRetry, TaskState};
use crate::event::{Event, EventType};

struct RetryableTask {
    base: Task,
    last_error: Option<String>,
}

impl RetryableTask {
    fn new(name: &str, max_retries: u32) -> Self {
        Self {
            base: TaskFactory::create_task_with_retries(
                name,
                EventType::Custom("retry".into()),
                max_retries,
            ),
            last_error: None,
        }
    }
}

impl TaskBehavior for RetryableTask {
    fn base(&self) -> &Task { &self.base }
    fn base_mut(&mut self) -> &mut Task { &mut self.base }
    
    fn handle_event(&mut self, event: &Event) {
        // Simulate random failures
        let will_fail = event.payload().contains("error");
        
        if will_fail {
            if self.should_retry() {
                self.base_mut().increment_retry();
                println!("[{}] Attempt {}/{}: Retrying...", 
                    self.base.name(), 
                    self.base.retry_count(), 
                    self.base.max_retries());
            } else {
                self.base_mut().set_state(TaskState::Failed);
                println!("[{}] Max retries exceeded, task failed", 
                    self.base.name());
            }
        } else {
            self.base_mut().reset_retry();
            println!("[{}] Success: {}", self.base.name(), event.payload());
        }
    }
}

impl TaskRetry for RetryableTask {
    fn on_retry(&mut self) -> Result<(), String> {
        println!("[{}] Retrying task...", self.base.name());
        Ok(())
    }

    fn should_retry(&self) -> bool {
        self.base.retry_count() < self.base.max_retries()
    }
}
```

---

## Example 5: Task-to-Task Message Passing

This example shows two tasks that communicate by scheduling messages for each other through a shared scheduler. It demonstrates how a task can send a message to a recipient task by targeting the recipient's event type.

```rust
use std::{cell::RefCell, rc::{Rc, Weak}};
use crate::event::{Event, EventType};
use crate::scheduler::EventScheduler;
use crate::task::{Task, TaskBehavior};

struct MessageTask {
    base: Task,
    scheduler: Weak<RefCell<EventScheduler>>,
}

impl MessageTask {
    fn new(name: impl Into<String>, scheduler: Weak<RefCell<EventScheduler>>) -> Self {
        let name = name.into();
        let event_type = EventType::Custom(name.clone());

        Self {
            base: Task::new(name, event_type),
            scheduler,
        }
    }

    fn send_message(&self, recipient: &str, payload: &str) {
        if let Some(scheduler) = self.scheduler.upgrade() {
            scheduler.borrow_mut().schedule(Event::new(
                EventType::Custom(recipient.to_string()),
                payload.to_string(),
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
        println!("[{}] received: {}", self.base.name(), event.payload());

        match (self.base.name(), event.payload()) {
            ("task_a", "start") => self.send_message("task_b", "ping"),
            ("task_b", "ping") => self.send_message("task_a", "pong"),
            ("task_a", "pong") => println!("[task_a] conversation complete."),
            _ => println!("[{}] no action for payload '{}'.", self.base.name(), event.payload()),
        }
    }
}

fn main() {
    let scheduler = Rc::new(RefCell::new(EventScheduler::new()));
    let task_a = MessageTask::new("task_a", Rc::downgrade(&scheduler));
    let task_b = MessageTask::new("task_b", Rc::downgrade(&scheduler));

    {
        let mut scheduler = scheduler.borrow_mut();
        scheduler.register_task(Box::new(task_a));
        scheduler.register_task(Box::new(task_b));
        scheduler.schedule(Event::new(EventType::Custom("task_a".into()), "start".into()));
    }

    scheduler.borrow_mut().dispatch();
}
```

---

## Example 6: Using the Scheduler

Complete example showing scheduler usage:

```rust
use crate::scheduler::EventScheduler;
use crate::event::{Event, EventType};

fn main() {
    let mut scheduler = EventScheduler::new();

    // Create and register tasks
    let db_task = DatabaseTask::new("db_handler");
    let process_task = ProcessingTask::new("processor");
    let config_task = ConfigurableTask::with_config("api", "INFO", "linear");

    scheduler.register_task(Box::new(db_task));
    scheduler.register_task(Box::new(process_task));
    scheduler.register_task(Box::new(config_task));

    println!("Scheduler initialized with {} tasks\n", 
        scheduler.registered_tasks());

    // Queue multiple events
    scheduler.schedule(Event::new(
        EventType::Custom("process".into()),
        "Item 1".into(),
    ));
    scheduler.schedule(Event::new(
        EventType::Custom("process".into()),
        "Item 2".into(),
    ));
    scheduler.schedule(Event::new(
        EventType::Custom("config".into()),
        "Configuration loaded".into(),
    ));

    println!("Queued {} events\n", scheduler.pending_events());

    // Dispatch all events
    scheduler.dispatch();

    println!("\nScheduler running: {}", scheduler.is_running());
    println!("Pending events: {}", scheduler.pending_events());

    // Unregister a task
    scheduler.unregister_task("processor");
    println!("After unregister: {} tasks remain", 
        scheduler.registered_tasks());
}
```

---

## Example 6: Custom EventHandler with Subscribers

Using the EventHandler for decoupled event handling:

```rust
use crate::event::{Event, EventHandler, EventType};

fn main() {
    let mut handler = EventHandler::new();

    // Subscribe multiple handlers to the same event type
    handler.subscribe(|event| {
        println!("[Logger] Event at {}: {}", 
            event.timestamp(), event.payload());
    });

    handler.subscribe(|event| {
        if event.payload().contains("error") {
            println!("[Alert] Error detected: {}", event.payload());
        }
    });

    handler.subscribe(|event| {
        println!("[Metrics] Event processed: {}", event.payload());
    });

    // Emit events
    let event1 = Event::new(
        EventType::Custom("app".into()),
        "Application started".into(),
    );
    handler.emit(&event1);

    let event2 = Event::new(
        EventType::Custom("error".into()),
        "Connection error occurred".into(),
    );
    handler.emit(&event2);
}
```

---

## Best Practices

1. **Always implement TaskBehavior**: This is the base trait that all tasks must implement
2. **Use TaskFactory**: Prefer factory methods over direct Task construction
3. **Implement lifecycle traits selectively**: Only implement traits your task actually needs
4. **Handle errors properly**: Return Result types from lifecycle methods
5. **Use properties for configuration**: Avoid adding fields for every configurable option
6. **Update task state**: Always update state in lifecycle methods
7. **Subscribe before dispatch**: Register all tasks before calling dispatcher

---

## Error Handling Pattern

```rust
// Always check Result types from lifecycle methods
match task.on_startup() {
    Ok(()) => println!("Task started successfully"),
    Err(e) => eprintln!("Failed to start task: {}", e),
}
```
