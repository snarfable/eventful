# Task Scheduler Enhancements

## Overview
This document describes the enhancements made to the event handler, task system, and scheduler while maintaining the existing code style and structure.

## Changes Made

### 1. Enhanced Event Handler (`event.rs`)

#### New Features:
- **EventHandler struct**: A generic event handler that supports subscribing to events and emitting them
- **Timestamp tracking**: Each event now includes a timestamp (Unix seconds)
- **Handler subscription**: Support for registering multiple handlers via the `subscribe` method

#### Key Additions:
```rust
pub struct EventHandler {
    handlers: Vec<Box<dyn Fn(&Event) + Send>>,
}

impl EventHandler {
    pub fn new() -> Self
    pub fn subscribe<F>(&mut self, handler: F) where F: Fn(&Event) + Send + 'static
    pub fn emit(&self, event: &Event)
}
```

---

### 2. Enhanced Task Structure (`task.rs`)

#### Task Lifecycle States:
```rust
pub enum TaskState {
    Initialized,  // Task created but not started
    Running,      // Task is currently executing
    Paused,       // Task is paused (can be resumed)
    Stopped,      // Task has been shut down
    Failed,       // Task encountered an error
}
```

#### Task Structure Enhancements:
- **State management**: Track task lifecycle state
- **Retry tracking**: Built-in retry counter and max retries configuration
- **Properties HashMap**: Store arbitrary key-value properties for tasks
- **Accessor methods**: Full API for managing all task properties

#### Key Methods:
- `set_state()`, `state()` - Manage task state
- `increment_retry()`, `reset_retry()` - Handle retry logic
- `set_property()`, `get_property()` - Manage custom properties
- `set_max_retries()`, `max_retries()` - Configure retry behavior

---

### 3. Task Lifecycle Traits

#### TaskStartup Trait
```rust
pub trait TaskStartup {
    fn on_startup(&mut self) -> Result<(), String>;
}
```
Implement this to define startup logic for a task.

#### TaskShutdown Trait
```rust
pub trait TaskShutdown {
    fn on_shutdown(&mut self) -> Result<(), String>;
}
```
Implement this to define cleanup logic when a task shuts down.

#### TaskRetry Trait
```rust
pub trait TaskRetry {
    fn on_retry(&mut self) -> Result<(), String>;
    fn should_retry(&self) -> bool;
}
```
Implement this to handle retry logic when a task fails.

#### TaskPausable Trait
```rust
pub trait TaskPausable {
    fn on_pause(&mut self) -> Result<(), String>;
    fn on_resume(&mut self) -> Result<(), String>;
    fn is_paused(&self) -> bool;
}
```
Implement this to support pausing and resuming tasks.

---

### 4. TaskFactory Pattern (`task.rs`)

The `TaskFactory` provides factory methods for creating tasks with different configurations:

```rust
pub struct TaskFactory;

impl TaskFactory {
    pub fn create_task(name, event_type) -> Task
    pub fn create_task_with_retries(name, event_type, max_retries) -> Task
    pub fn create_task_with_properties(name, event_type, properties) -> Task
}
```

#### Usage Examples:
```rust
// Basic task
let task = TaskFactory::create_task("my_task", EventType::Custom("event".into()));

// Task with retry configuration
let task = TaskFactory::create_task_with_retries("task_name", event_type, 5);

// Task with properties
let mut props = HashMap::new();
props.insert("key".to_string(), "value".to_string());
let task = TaskFactory::create_task_with_properties("task", event_type, props);
```

---

### 5. Enhanced Scheduler (`scheduler.rs`)

#### New Methods:
- `unregister_task(name)` - Remove tasks by name
- `is_running()` - Check if scheduler is currently dispatching
- `pending_events()` - Get count of queued events
- `registered_tasks()` - Get count of registered tasks
- `clear_queue()` - Clear pending events

#### Enhanced State:
- `running` flag to track scheduler state

---

### 6. Updated Example Usage (`main.rs`)

The example demonstrates:

1. **TaskFactory usage**: Creating tasks using factory methods
2. **Lifecycle trait implementation**: PrintTask now implements TaskStartup, TaskShutdown, and TaskPausable
3. **Properties management**: Creating tasks with custom properties
4. **Enhanced logging**: Shows task state in event handling
5. **Scheduler API**: Uses new scheduler methods for introspection

#### Example Features:
```rust
impl TaskStartup for PrintTask {
    fn on_startup(&mut self) -> Result<(), String> {
        println!("Starting task: {}", self.base.name());
        self.base_mut().set_state(TaskState::Running);
        Ok(())
    }
}

impl TaskPausable for PrintTask {
    fn on_pause(&mut self) -> Result<(), String> {
        println!("Pausing task: {}", self.base.name());
        self.base_mut().set_state(TaskState::Paused);
        Ok(())
    }
    
    fn on_resume(&mut self) -> Result<(), String> {
        println!("Resuming task: {}", self.base.name());
        self.base_mut().set_state(TaskState::Running);
        Ok(())
    }
    
    fn is_paused(&self) -> bool {
        self.base().state() == TaskState::Paused
    }
}
```

---

## Architecture Benefits

1. **Composition over Inheritance**: Tasks use trait composition for flexible behavior
2. **Factory Pattern**: Simplifies task creation with common configurations
3. **Type Safety**: Rust's type system prevents invalid state transitions
4. **Extensibility**: New traits can be implemented for custom behavior
5. **Property-driven Configuration**: Tasks can store arbitrary configuration without struct modifications
6. **Error Handling**: All lifecycle methods return `Result` for proper error management

---

## Integration Guide

To use these enhancements in your code:

1. **Create a custom task type**:
   ```rust
   struct MyTask {
       base: Task,
       // custom fields
   }
   ```

2. **Implement TaskBehavior**:
   ```rust
   impl TaskBehavior for MyTask {
       fn base(&self) -> &Task { &self.base }
       fn base_mut(&mut self) -> &mut Task { &mut self.base }
       fn handle_event(&mut self, event: &Event) { /* ... */ }
   }
   ```

3. **Optionally implement lifecycle traits**:
   ```rust
   impl TaskStartup for MyTask { /* ... */ }
   impl TaskShutdown for MyTask { /* ... */ }
   impl TaskPausable for MyTask { /* ... */ }
   ```

4. **Register with scheduler**:
   ```rust
   let mut scheduler = EventScheduler::new();
   scheduler.register_task(Box::new(my_task));
   ```

---

## Code Style & Patterns

All enhancements maintain the existing codebase style:
- Simple, readable method names
- Clear separation of concerns
- Traits for extensibility
- Factory pattern for object creation
- Generic types where appropriate
- Proper error handling with Result types
