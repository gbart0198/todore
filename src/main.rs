use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fs;
use std::io;
use std::str::FromStr;

trait Formatter {
    fn format(&self, tasks: &TaskList) -> Result<String, Box<dyn std::error::Error>>;
}

struct PlaintextFormatter;

impl Formatter for PlaintextFormatter {
    fn format(&self, tasks: &TaskList) -> Result<String, Box<dyn std::error::Error>> {
        Ok(tasks
            .tasks
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
    fn format(&self, tasks: &TaskList) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(tasks)?)
    }
}

struct YamlFormatter;

impl Formatter for YamlFormatter {
    fn format(&self, tasks: &TaskList) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(tasks)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct TaskList {
    tasks: Vec<Task>,
}
impl TaskList {
    fn new() -> Self {
        TaskList { tasks: vec![] }
    }

    fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    fn remove(&mut self, task_id: u32) {
        self.tasks.retain(|task| task.id != task_id);
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

    fn export_to_string<T: Formatter>(
        &self,
        formatter: &dyn Formatter,
    ) -> Result<String, Box<dyn std::error::Error>> {
        formatter.format(self)
    }

    fn import(&mut self, tasks: &str) -> Result<(), Box<dyn std::error::Error>> {
        let imported: TaskList = serde_json::from_str(tasks)?;
        self.tasks = imported.tasks;
        Ok(())
    }
}

#[derive(Debug)]
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
        out_file: String,
    },
    Quit,
}

#[derive(Debug)]
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

#[derive(Debug)]
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
                if parts.len() < 3 {
                    return Err("Invalid arguments for export.".into());
                }
                let format = Format::from_str(parts[1])?;
                Ok(Command::Export {
                    format,
                    out_file: parts[2].into(),
                })
            }
            _ => Err("Invalid argument.".into()),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task_list = TaskList::new();
    println!("Welcome to the Todore in-memory TODO list!");

    // lets load the tasks from a file in a shared location, if it exists
    // for testing purposes, lets make this file directly under the pwd
    let tasks_file = "tasks.json";
    let existing_tasks = fs::read_to_string(tasks_file)?;
    task_list.import(&existing_tasks)?;

    let mut input = String::new();
    let jf = JsonFormatter::new();
    let yf = YamlFormatter::new();
    let ptf = PlaintextFormatter::new();
    let mut counter = 0;
    loop {
        if !task_list.tasks.is_empty() {
            println!("Here are your current tasks:");
            println!("{}", task_list.export_to_string::<JsonFormatter>(&jf)?);
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
                task_list.add(Task::new(counter, val));
                counter += 1;
            }
            Command::Remove { id } => task_list.remove(id),
            Command::Update { id, new_val, field } => match field {
                TaskField::Description => task_list.update_description(id, new_val)?,
                TaskField::Status => {
                    task_list.update_status(id, TaskStatus::from_str(&new_val)?)?
                }
            },
            Command::Quit => break,
            Command::Export { format, out_file } => match format {
                Format::Json => {
                    let content = task_list.export_to_string::<JsonFormatter>(&jf)?;
                    fs::write(out_file, content)?;
                }
                Format::Yaml => {
                    let content = task_list.export_to_string::<YamlFormatter>(&yf)?;
                    fs::write(out_file, content)?;
                }
                Format::Plaintext => {
                    let content = task_list.export_to_string::<PlaintextFormatter>(&ptf)?;
                    fs::write(out_file, content)?;
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

        list.add(Task::new(1, "Test".into()));
        assert!(matches!(list.tasks.len(), 1));

        list.update_status(1, TaskStatus::InProgress).unwrap();
        assert!(matches!(list.tasks[0].status, TaskStatus::InProgress));
    }

    #[test]
    fn test_update_description() {
        let mut list = TaskList::new();

        list.add(Task::new(2, "Test2".into()));
        let new_description = "Test123";

        list.update_description(2, new_description.into()).unwrap();
        assert_eq!(list.tasks[0].description, new_description);
    }

    #[test]
    fn test_remove() {
        let mut list = TaskList::new();

        list.add(Task::new(1, "Test1".into()));
        assert_eq!(list.tasks.len(), 1);

        list.remove(1);
        assert_eq!(list.tasks.len(), 0);
    }

    #[test]
    fn test_task_new() {
        let task = Task::new(42, "Test task".to_string());
        assert_eq!(task.id, 42);
        assert_eq!(task.description, "Test task");
        assert!(matches!(task.status, TaskStatus::NotStarted));
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new(1, "Test task".to_string());
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, task.id);
        assert_eq!(deserialized.description, task.description);
        assert!(matches!(deserialized.status, TaskStatus::NotStarted));
    }

    // TaskStatus enum tests
    #[test]
    fn test_taskstatus_fromstr_valid() {
        assert!(matches!(
            TaskStatus::from_str("not started"),
            Ok(TaskStatus::NotStarted)
        ));
        assert!(matches!(
            TaskStatus::from_str("ns"),
            Ok(TaskStatus::NotStarted)
        ));
        assert!(matches!(
            TaskStatus::from_str("in progress"),
            Ok(TaskStatus::InProgress)
        ));
        assert!(matches!(
            TaskStatus::from_str("ip"),
            Ok(TaskStatus::InProgress)
        ));
        assert!(matches!(
            TaskStatus::from_str("completed"),
            Ok(TaskStatus::Completed)
        ));
        assert!(matches!(
            TaskStatus::from_str("c"),
            Ok(TaskStatus::Completed)
        ));
    }

    #[test]
    fn test_taskstatus_fromstr_invalid() {
        assert!(TaskStatus::from_str("invalid").is_err());
        assert!(TaskStatus::from_str("").is_err());
        assert!(TaskStatus::from_str("notstarted").is_err());
        assert!(TaskStatus::from_str("done").is_err());
    }

    #[test]
    fn test_taskstatus_display() {
        assert_eq!(format!("{}", TaskStatus::NotStarted), "Not Started");
        assert_eq!(format!("{}", TaskStatus::InProgress), "In Progress");
        assert_eq!(format!("{}", TaskStatus::Completed), "Completed");
    }

    #[test]
    fn test_taskstatus_serialization() {
        let statuses = vec![
            TaskStatus::NotStarted,
            TaskStatus::InProgress,
            TaskStatus::Completed,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
            assert!(matches!(
                (status, deserialized),
                (TaskStatus::NotStarted, TaskStatus::NotStarted)
                    | (TaskStatus::InProgress, TaskStatus::InProgress)
                    | (TaskStatus::Completed, TaskStatus::Completed)
            ));
        }
    }

    // TaskList struct tests
    #[test]
    fn test_tasklist_new() {
        let list = TaskList::new();
        assert_eq!(list.tasks.len(), 0);
    }

    #[test]
    fn test_tasklist_add() {
        let mut list = TaskList::new();
        let task = Task::new(1, "Test task".to_string());
        list.add(task);
        assert_eq!(list.tasks.len(), 1);
        assert_eq!(list.tasks[0].id, 1);
        assert_eq!(list.tasks[0].description, "Test task");
    }

    #[test]
    fn test_tasklist_update_status_nonexistent() {
        let mut list = TaskList::new();
        let result = list.update_status(999, TaskStatus::Completed);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Task with id 999 was not found");
    }

    #[test]
    fn test_tasklist_update_description_nonexistent() {
        let mut list = TaskList::new();
        let result = list.update_description(999, "New description".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Task with id 999 was not found");
    }

    #[test]
    fn test_tasklist_export_import() {
        let mut list = TaskList::new();
        list.add(Task::new(1, "Task 1".to_string()));
        list.add(Task::new(2, "Task 2".to_string()));

        // Export to JSON
        let json_formatter = JsonFormatter::new();
        let json_str = list
            .export_to_string::<JsonFormatter>(&json_formatter)
            .unwrap();

        // Import into new list
        let mut new_list = TaskList::new();
        new_list.import(&json_str).unwrap();

        // Verify import worked
        assert_eq!(new_list.tasks.len(), 2);
        assert_eq!(new_list.tasks[0].id, 1);
        assert_eq!(new_list.tasks[0].description, "Task 1");
        assert_eq!(new_list.tasks[1].id, 2);
        assert_eq!(new_list.tasks[1].description, "Task 2");
    }

    #[test]
    fn test_tasklist_import_invalid_json() {
        let mut list = TaskList::new();
        let result = list.import("invalid json");
        assert!(result.is_err());
    }

    // Formatter implementation tests
    #[test]
    fn test_plaintext_formatter() {
        let mut list = TaskList::new();
        list.add(Task::new(1, "Task 1".to_string()));
        list.add(Task::new(2, "Task 2".to_string()));

        let formatter = PlaintextFormatter::new();
        let result = formatter.format(&list).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("1: Task 1"));
        assert!(lines[1].contains("2: Task 2"));
    }

    #[test]
    fn test_json_formatter() {
        let mut list = TaskList::new();
        list.add(Task::new(1, "Test task".to_string()));

        let formatter = JsonFormatter::new();
        let result = formatter.format(&list).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["tasks"][0]["id"], 1);
        assert_eq!(parsed["tasks"][0]["description"], "Test task");
    }

    #[test]
    fn test_yaml_formatter() {
        let mut list = TaskList::new();
        list.add(Task::new(1, "Test task".to_string()));

        let formatter = YamlFormatter::new();
        let result = formatter.format(&list).unwrap();

        // Should be valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();
        assert_eq!(parsed["tasks"][0]["id"].as_u64().unwrap(), 1);
        assert_eq!(
            parsed["tasks"][0]["description"].as_str().unwrap(),
            "Test task"
        );
    }

    // Command parsing tests
    #[test]
    fn test_command_add() {
        let cmd = Command::from_str("add Buy groceries").unwrap();
        match cmd {
            Command::Add { val } => assert_eq!(val, "Buy"),
            _ => panic!("Expected Add command"),
        }

        let cmd_short = Command::from_str("a Buy groceries").unwrap();
        match cmd_short {
            Command::Add { val } => assert_eq!(val, "Buy"),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_command_add_insufficient_args() {
        let result = Command::from_str("add");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid arguments for add."
        );
    }

    #[test]
    fn test_command_remove() {
        let cmd = Command::from_str("remove 5").unwrap();
        match cmd {
            Command::Remove { id } => assert_eq!(id, 5),
            _ => panic!("Expected Remove command"),
        }

        let cmd_short = Command::from_str("r 10").unwrap();
        match cmd_short {
            Command::Remove { id } => assert_eq!(id, 10),
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_command_remove_insufficient_args() {
        let result = Command::from_str("remove");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid arguments for remove."
        );
    }

    #[test]
    fn test_command_remove_invalid_id() {
        let result = Command::from_str("remove abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_update_status() {
        let cmd = Command::from_str("update 1 status completed").unwrap();
        match cmd {
            Command::Update { id, new_val, field } => {
                assert_eq!(id, 1);
                assert_eq!(new_val, "completed");
                assert!(matches!(field, TaskField::Status));
            }
            _ => panic!("Expected Update command"),
        }

        let cmd_short = Command::from_str("u 2 s ip").unwrap();
        match cmd_short {
            Command::Update { id, new_val, field } => {
                assert_eq!(id, 2);
                assert_eq!(new_val, "ip");
                assert!(matches!(field, TaskField::Status));
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_command_update_description() {
        let cmd = Command::from_str("update 1 description New description").unwrap();
        match cmd {
            Command::Update { id, new_val, field } => {
                assert_eq!(id, 1);
                assert_eq!(new_val, "New");
                assert!(matches!(field, TaskField::Description));
            }
            _ => panic!("Expected Update command"),
        }

        let cmd_short = Command::from_str("u 2 d Short desc").unwrap();
        match cmd_short {
            Command::Update { id, new_val, field } => {
                assert_eq!(id, 2);
                assert_eq!(new_val, "Short");
                assert!(matches!(field, TaskField::Description));
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_command_update_insufficient_args() {
        let result = Command::from_str("update 1 status");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid arguments for update."
        );
    }

    #[test]
    fn test_command_update_invalid_field() {
        let result = Command::from_str("update 1 invalid completed");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_export() {
        let cmd = Command::from_str("export json output.json").unwrap();
        match cmd {
            Command::Export { format, out_file } => {
                assert!(matches!(format, Format::Json));
                assert_eq!(out_file, "output.json");
            }
            _ => panic!("Expected Export command"),
        }

        let cmd_short = Command::from_str("e y output.yaml").unwrap();
        match cmd_short {
            Command::Export { format, out_file } => {
                assert!(matches!(format, Format::Yaml));
                assert_eq!(out_file, "output.yaml");
            }
            _ => panic!("Expected Export command"),
        }

        let cmd_plaintext = Command::from_str("e p output.txt").unwrap();
        match cmd_plaintext {
            Command::Export { format, out_file } => {
                assert!(matches!(format, Format::Plaintext));
                assert_eq!(out_file, "output.txt");
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_command_export_insufficient_args() {
        let result = Command::from_str("export json");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid arguments for export."
        );
    }

    #[test]
    fn test_command_export_invalid_format() {
        let result = Command::from_str("export invalid output.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_quit() {
        let cmd = Command::from_str("quit").unwrap();
        assert!(matches!(cmd, Command::Quit));

        let cmd_short = Command::from_str("q").unwrap();
        assert!(matches!(cmd_short, Command::Quit));
    }

    #[test]
    fn test_command_invalid() {
        let result = Command::from_str("invalid command");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid argument.");
    }

    // Format and TaskField enum parsing tests
    #[test]
    fn test_format_fromstr_valid() {
        assert!(matches!(Format::from_str("json"), Ok(Format::Json)));
        assert!(matches!(Format::from_str("j"), Ok(Format::Json)));
        assert!(matches!(Format::from_str("yaml"), Ok(Format::Yaml)));
        assert!(matches!(Format::from_str("y"), Ok(Format::Yaml)));
        assert!(matches!(
            Format::from_str("plaintext"),
            Ok(Format::Plaintext)
        ));
        assert!(matches!(Format::from_str("p"), Ok(Format::Plaintext)));
    }

    #[test]
    fn test_format_fromstr_invalid() {
        assert!(Format::from_str("invalid").is_err());
        assert!(Format::from_str("").is_err());
        assert!(Format::from_str("txt").is_err());
    }

    #[test]
    fn test_taskfield_fromstr_valid() {
        assert!(matches!(
            TaskField::from_str("description"),
            Ok(TaskField::Description)
        ));
        assert!(matches!(
            TaskField::from_str("d"),
            Ok(TaskField::Description)
        ));
        assert!(matches!(
            TaskField::from_str("status"),
            Ok(TaskField::Status)
        ));
        assert!(matches!(TaskField::from_str("s"), Ok(TaskField::Status)));
    }

    #[test]
    fn test_taskfield_fromstr_invalid() {
        assert!(TaskField::from_str("invalid").is_err());
        assert!(TaskField::from_str("").is_err());
        assert!(TaskField::from_str("name").is_err());
    }
}
