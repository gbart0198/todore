use serde::Serialize;
use std::fmt;
use std::io;
use std::str::FromStr;

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

impl FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "not started" | "ns" => Ok(TaskStatus::NotStarted),
            "in progress" | "ip" => Ok(TaskStatus::InProgress),
            "completed" | "c" => Ok(TaskStatus::Completed),
            _ => Err("Error while parsing task status".into()),
        }
    }
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
    fn new() -> Self {
        TaskList { tasks: vec![] }
    }

    fn add(&mut self, task: Task) -> Result<(), String> {
        self.tasks.push(task);
        Ok(())
    }

    fn remove(&mut self, task_id: u32) -> Result<(), String> {
        self.tasks.retain(|task| task.id != task_id);

        Ok(())
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

    fn export<T: Formatter>(
        &self,
        formatter: &dyn Formatter,
    ) -> Result<String, Box<dyn std::error::Error>> {
        formatter.format(&self.tasks)
    }
}

enum Command {
    Add {
        val: String,
    },
    Remove {
        id: u32,
    },
    Update {
        id: u32,
        new_val: String,
        field: TaskField,
    },
    Export {
        format: Format,
    },
    Quit,
}

enum Format {
    Json,
    Yaml,
    Plaintext,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "j" | "json" => Ok(Format::Json),
            "y" | "yaml" => Ok(Format::Yaml),
            "p" | "plaintext" => Ok(Format::Plaintext),
            _ => Err("Invalid export format.".into()),
        }
    }
}

enum TaskField {
    Description,
    Status,
}

impl FromStr for TaskField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "description" | "d" => Ok(TaskField::Description),
            "status" | "s" => Ok(TaskField::Status),
            _ => Err("Invalid field argument".into()),
        }
    }
}

impl Command {
    fn from_str(val: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<_> = val.split(" ").collect();
        match parts[0] {
            "a" | "add" => {
                if parts.len() < 2 {
                    return Err("Invalid arguments for add.".into());
                }
                let val = parts[1].into();
                Ok(Command::Add { val })
            }
            "r" | "remove" => {
                if parts.len() < 2 {
                    return Err("Invalid arguments for remove.".into());
                }
                let id = parts[1].parse::<u32>()?;
                Ok(Command::Remove { id })
            }
            "u" | "update" => {
                if parts.len() < 4 {
                    return Err("Invalid arguments for update.".into());
                }
                let id = parts[1].parse::<u32>()?;
                let field = TaskField::from_str(&parts[2].to_lowercase())?;
                let new_val = parts[3].into();

                Ok(Command::Update { id, new_val, field })
            }
            "q" | "quit" => Ok(Command::Quit),
            "e" | "export" => {
                if parts.len() < 2 {
                    return Err("Invalid arguments for export.".into());
                }
                let format = Format::from_str(parts[1])?;
                Ok(Command::Export { format })
            }
            _ => Err("Invalid argument.".into()),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task_list = TaskList::new();
    println!("Welcome to the Todore in-memory TODO list!");

    let mut input = String::new();
    let jf = JsonFormatter::new();
    let yf = YamlFormatter::new();
    let ptf = PlaintextFormatter::new();
    let mut counter = 0;
    loop {
        if !task_list.tasks.is_empty() {
            println!("Here are your current tasks:");
            println!("{}", task_list.export::<JsonFormatter>(&jf)?);
        }
        println!("Below are the options:");
        println!("[a | add] <TODO-item>");
        println!("[r | remove] <TODO-item-id>");
        println!("[u | update] <TODO-item-id> [s | status] | [d | description] <new-value>");
        println!("[e | export] [j | json] | [y | yaml] | [p | plaintext]");
        println!("[q | quit]");

        io::stdin().read_line(&mut input)?;

        println!("You chose: {}", input.trim());
        let command = Command::from_str(&input.trim().to_lowercase())?;
        match command {
            Command::Add { val } => {
                task_list.add(Task::new(counter, val))?;
                counter += 1;
            }
            Command::Remove { id } => task_list.remove(id)?,
            Command::Update { id, new_val, field } => match field {
                TaskField::Description => task_list.update_description(id, new_val)?,
                TaskField::Status => {
                    task_list.update_status(id, TaskStatus::from_str(&new_val)?)?
                }
            },
            Command::Quit => break,
            Command::Export { format } => match format {
                Format::Json => {
                    println!("{}", task_list.export::<JsonFormatter>(&jf)?);
                }
                Format::Yaml => {
                    println!("{}", task_list.export::<YamlFormatter>(&yf)?);
                }
                Format::Plaintext => {
                    println!("{}", task_list.export::<PlaintextFormatter>(&ptf)?);
                }
            },
        }

        input.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_status() {
        let mut list = TaskList::new();

        list.add(Task::new(1, "Test".into())).unwrap();
        assert!(matches!(list.tasks.len(), 1));

        list.update_status(1, TaskStatus::InProgress).unwrap();
        assert!(matches!(list.tasks[0].status, TaskStatus::InProgress));
    }

    #[test]
    fn test_update_description() {
        let mut list = TaskList::new();

        list.add(Task::new(2, "Test2".into())).unwrap();
        let new_description = "Test123";

        list.update_description(2, new_description.into()).unwrap();
        assert_eq!(list.tasks[0].description, new_description);
    }

    #[test]
    fn test_remove() {
        let mut list = TaskList::new();

        list.add(Task::new(1, "Test1".into())).unwrap();
        assert_eq!(list.tasks.len(), 1);

        list.remove(1).unwrap();
        assert_eq!(list.tasks.len(), 0);
    }
}
