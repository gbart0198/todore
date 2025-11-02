use serde::Serialize;
use std::fmt;

trait Formatter {
    fn format(&self, tasks: &[Task]) -> Result<String, Box<dyn std::error::Error>>;
}

struct PlaintextFormatter;

impl Formatter for PlaintextFormatter {
    fn format(&self, tasks: &[Task]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(tasks
            .iter()
            .map(|task| format!("{}: {}\t{}", task.id, task.description, task.status))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

impl PlaintextFormatter {
    fn new() -> Self {
        Self
    }
}
impl JsonFormatter {
    fn new() -> Self {
        Self
    }
}
impl YamlFormatter {
    fn new() -> Self {
        Self
    }
}

struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, tasks: &[Task]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(tasks)?)
    }
}

struct YamlFormatter;

impl Formatter for YamlFormatter {
    fn format(&self, tasks: &[Task]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(tasks)?)
    }
}

#[derive(Debug, Serialize)]
struct Task {
    id: u32,
    description: String,
    status: TaskStatus,
}

impl Task {
    fn new(id: u32, description: String) -> Self {
        Self {
            id,
            description,
            status: TaskStatus::NotStarted,
        }
    }
}

#[derive(Debug, Serialize)]
enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskStatus::NotStarted => "Not Started",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Completed => "Completed",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize)]
struct TaskList {
    tasks: Vec<Task>,
}
impl TaskList {
    fn add(&mut self, task: Task) -> Result<(), String> {
        self.tasks.push(task);
        Ok(())
    }

    fn new() -> Self {
        TaskList { tasks: vec![] }
    }

    fn update_status(&mut self, task_id: u32, new_status: TaskStatus) -> Result<(), String> {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) {
            task.status = new_status;
            Ok(())
        } else {
            Err(format!("Task with id {} was not found", task_id))
        }
    }
    fn update_description(&mut self, task_id: u32, new_description: String) -> Result<(), String> {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == task_id) {
            task.description = new_description;
            Ok(())
        } else {
            Err(format!("Task with id {} was not found", task_id))
        }
    }

    fn export<T: Formatter>(&self, formatter: &T) -> Result<String, Box<dyn std::error::Error>> {
        formatter.format(&self.tasks)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = TaskList::new();
    let task1 = Task::new(1, "Testing task".into());
    let task2 = Task::new(2, "Testing task 2".into());
    tasks.add(task1)?;
    tasks.add(task2)?;

    tasks.update_status(1, TaskStatus::InProgress)?;
    tasks.update_status(2, TaskStatus::Completed)?;
    tasks.update_description(1, "New description!".into())?;

    let json_f = JsonFormatter::new();
    let pt_f = PlaintextFormatter::new();
    let yaml_f = YamlFormatter::new();

    println!("{}", tasks.export(&json_f)?);
    println!("{}", tasks.export(&pt_f)?);
    println!("{}", tasks.export(&yaml_f)?);
    Ok(())
}
