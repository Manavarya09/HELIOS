use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOperation {
    List(PathBuf),
    ChangeDirectory(PathBuf),
    CurrentDirectory,
    ReadFile(PathBuf),
    WriteFile(PathBuf, String),
    CreateDirectory(PathBuf),
    Delete(PathBuf),
}

impl FileOperation {
    pub fn execute(&self) -> Result<String, String> {
        match self {
            FileOperation::List(path) => Self::list_directory(path),
            FileOperation::ChangeDirectory(path) => Self::change_directory(path),
            FileOperation::CurrentDirectory => Self::current_directory(),
            FileOperation::ReadFile(path) => Self::read_file(path),
            FileOperation::WriteFile(path, content) => Self::write_file(path, content),
            FileOperation::CreateDirectory(path) => Self::create_directory(path),
            FileOperation::Delete(path) => Self::delete(path),
        }
    }

    fn list_directory(path: &PathBuf) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        let entries =
            std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut output = Vec::new();
        output.push(format!("Contents of {}:", path.display()));
        output.push("------------------------------------------------".to_string());

        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.path().is_dir() {
                "[DIR]"
            } else {
                "[FILE]"
            };
            output.push(format!("{}  {}", file_type, file_name));
        }

        Ok(output.join("\n"))
    }

    fn change_directory(path: &PathBuf) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", path.display()));
        }
        if !path.is_dir() {
            return Err(format!("Not a directory: {}", path.display()));
        }
        std::env::set_current_dir(path)
            .map_err(|e| format!("Failed to change directory: {}", e))?;
        Ok(format!("Changed directory to: {}", path.display()))
    }

    fn current_directory() -> Result<String, String> {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get current directory: {}", e))
    }

    fn read_file(path: &PathBuf) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    fn write_file(path: &PathBuf, content: &str) -> Result<String, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(format!("File written: {}", path.display()))
    }

    fn create_directory(path: &PathBuf) -> Result<String, String> {
        std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))?;
        Ok(format!("Directory created: {}", path.display()))
    }

    fn delete(path: &PathBuf) -> Result<String, String> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("Failed to delete directory: {}", e))?;
        } else {
            std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }
        Ok(format!("Deleted: {}", path.display()))
    }
}

pub fn parse_file_command(args: &[&str]) -> Result<FileOperation, String> {
    if args.is_empty() {
        return Err("Usage: ls | cd <path> | pwd | read <file> | write <file> <content> | mkdir <dir> | delete <path>".to_string());
    }

    match args[0] {
        "ls" => {
            let path = if args.len() > 1 {
                PathBuf::from(args[1])
            } else {
                std::env::current_dir().unwrap_or_default()
            };
            Ok(FileOperation::List(path))
        }
        "cd" => {
            if args.len() < 2 {
                return Err("Usage: cd <path>".to_string());
            }
            Ok(FileOperation::ChangeDirectory(PathBuf::from(args[1])))
        }
        "pwd" => Ok(FileOperation::CurrentDirectory),
        "read" => {
            if args.len() < 2 {
                return Err("Usage: read <file>".to_string());
            }
            Ok(FileOperation::ReadFile(PathBuf::from(args[1])))
        }
        "write" => {
            if args.len() < 3 {
                return Err("Usage: write <file> <content>".to_string());
            }
            Ok(FileOperation::WriteFile(
                PathBuf::from(args[1]),
                args[2..].join(" "),
            ))
        }
        "mkdir" => {
            if args.len() < 2 {
                return Err("Usage: mkdir <directory>".to_string());
            }
            Ok(FileOperation::CreateDirectory(PathBuf::from(args[1])))
        }
        "delete" | "rm" => {
            if args.len() < 2 {
                return Err("Usage: delete <path>".to_string());
            }
            Ok(FileOperation::Delete(PathBuf::from(args[1])))
        }
        _ => Err(format!("Unknown file command: {}", args[0])),
    }
}
