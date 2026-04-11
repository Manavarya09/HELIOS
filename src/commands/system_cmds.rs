use std::str::FromStr;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub enum SystemCommand {
    ListProcesses(usize),
    KillProcess(String),
    ProcessInfo(String),
}

impl SystemCommand {
    pub fn execute(&self, system: &mut System) -> Result<String, String> {
        match self {
            SystemCommand::ListProcesses(count) => Self::list_processes(system, *count),
            SystemCommand::KillProcess(pid_str) => Self::kill_process(system, pid_str),
            SystemCommand::ProcessInfo(pid_str) => Self::process_info(system, pid_str),
        }
    }

    fn list_processes(system: &mut System, count: usize) -> Result<String, String> {
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut processes: Vec<_> = system
            .processes()
            .iter()
            .map(|(pid, process)| {
                (
                    pid,
                    process.name().to_string_lossy().to_string(),
                    process.cpu_usage(),
                    process.memory(),
                )
            })
            .collect();

        processes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let count = count.min(50).max(1);
        let processes: Vec<_> = processes.into_iter().take(count).collect();

        let mut output = Vec::new();
        output.push(format!(
            "{:<10} {:<30} {:>10} {:>15}",
            "PID", "Name", "CPU%", "Memory (MB)"
        ));
        output.push(
            "------------------------------------------------------------------------".to_string(),
        );

        for (pid, name, cpu, memory) in processes {
            let mem_mb = memory / 1024 / 1024;
            output.push(format!(
                "{:<10} {:<30} {:>9.1}% {:>14}",
                pid.as_u32(),
                name,
                cpu,
                mem_mb
            ));
        }

        Ok(output.join("\n"))
    }

    fn kill_process(system: &mut System, pid_str: &str) -> Result<String, String> {
        let pid = Pid::from_str(pid_str).map_err(|_| "Invalid PID format".to_string())?;

        system.refresh_processes(ProcessesToUpdate::All, true);

        if let Some(process) = system.process(pid) {
            if process.kill() {
                Ok(format!("Process {} killed successfully", pid_str))
            } else {
                Err(format!("Failed to kill process {}", pid_str))
            }
        } else {
            Err(format!("Process {} not found", pid_str))
        }
    }

    fn process_info(system: &mut System, pid_str: &str) -> Result<String, String> {
        let pid = Pid::from_str(pid_str).map_err(|_| "Invalid PID format".to_string())?;

        system.refresh_processes(ProcessesToUpdate::All, true);

        if let Some(process) = system.process(pid) {
            let mut output = Vec::new();
            output.push(format!("Process Information for PID: {}", pid_str));
            output.push("-----------------------------------------------".to_string());
            output.push(format!("Name: {}", process.name().to_string_lossy()));
            output.push(format!("PID: {}", process.pid()));
            output.push(format!("CPU Usage: {:.1}%", process.cpu_usage()));
            output.push(format!("Memory: {} MB", process.memory() / 1024 / 1024));
            output.push(format!("Status: {:?}", process.status()));

            if let Some(cmd) = process.cmd().first() {
                output.push(format!("Command: {}", cmd.to_string_lossy()));
            }

            Ok(output.join("\n"))
        } else {
            Err(format!("Process {} not found", pid_str))
        }
    }
}

pub fn parse_system_command(args: &[&str]) -> Result<SystemCommand, String> {
    if args.is_empty() {
        return Err("Usage: processes [count] | kill <pid> | info <pid>".to_string());
    }

    match args[0] {
        "processes" | "ps" => {
            let count: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10);
            Ok(SystemCommand::ListProcesses(count))
        }
        "kill" => {
            if args.len() < 2 {
                return Err("Usage: kill <pid>".to_string());
            }
            Ok(SystemCommand::KillProcess(args[1].to_string()))
        }
        "info" => {
            if args.len() < 2 {
                return Err("Usage: info <pid>".to_string());
            }
            Ok(SystemCommand::ProcessInfo(args[1].to_string()))
        }
        _ => Err(format!("Unknown system command: {}", args[0])),
    }
}
