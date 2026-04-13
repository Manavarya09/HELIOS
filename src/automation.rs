use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScheduler {
    pub tasks: Vec<ScheduledTask>,
    pub task_history: VecDeque<TaskHistoryEntry>,
    pub is_running: bool,
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub command: String,
    pub schedule_type: ScheduleType,
    pub interval_seconds: u64,
    pub next_run: u64,
    pub enabled: bool,
    pub last_result: Option<TaskResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScheduleType {
    Once,
    Interval,
    Daily,
    Weekly,
    Cron,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub timestamp: u64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEntry {
    pub task_id: String,
    pub task_name: String,
    pub result: TaskResult,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            task_history: VecDeque::new(),
            is_running: false,
            max_history: 100,
        }
    }

    pub fn add_task(
        &mut self,
        name: String,
        command: String,
        schedule: ScheduleType,
        interval: u64,
    ) -> String {
        let id = format!("task_{}", self.tasks.len() + 1);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.tasks.push(ScheduledTask {
            id: id.clone(),
            name,
            command,
            schedule_type: schedule,
            interval_seconds: interval,
            next_run: now + interval,
            enabled: true,
            last_result: None,
        });

        id
    }

    pub fn remove_task(&mut self, id: &str) -> bool {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn enable_task(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_task(&mut self, id: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn list_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.iter().collect()
    }

    pub fn get_task(&self, id: &str) -> Option<&ScheduledTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn process_due_tasks(&mut self) -> Vec<&ScheduledTask> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.tasks
            .iter()
            .filter(|t| t.enabled && t.next_run <= now)
            .collect()
    }

    pub fn record_result(&mut self, task_id: &str, result: TaskResult) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.last_result = Some(result.clone());
            task.next_run = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + task.interval_seconds;
        }

        if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
            self.task_history.push_front(TaskHistoryEntry {
                task_id: task_id.to_string(),
                task_name: task.name.clone(),
                result,
            });

            if self.task_history.len() > self.max_history {
                self.task_history.pop_back();
            }
        }
    }

    pub fn get_summary(&self) -> String {
        let total = self.tasks.len();
        let enabled = self.tasks.iter().filter(|t| t.enabled).count();
        let disabled = total - enabled;
        let running = if self.is_running { "ACTIVE" } else { "IDLE" };

        format!(
            "Scheduler: {} | Tasks: {} | Enabled: {} | Disabled: {}",
            running, total, enabled, disabled
        )
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSystem {
    pub macros: Vec<Macro>,
    pub is_recording: bool,
    pub current_recording: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub trigger: String,
    pub actions: Vec<MacroAction>,
    pub enabled: bool,
    pub use_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroAction {
    Command(String),
    Wait(u64),
    Type(String),
    PressKey(String),
}

impl MacroSystem {
    pub fn new() -> Self {
        Self {
            macros: Vec::new(),
            is_recording: false,
            current_recording: None,
        }
    }

    pub fn create_macro(&mut self, name: String, trigger: String) -> String {
        let id = format!("macro_{}", self.macros.len() + 1);

        self.macros.push(Macro {
            id: id.clone(),
            name,
            trigger,
            actions: Vec::new(),
            enabled: true,
            use_count: 0,
        });

        id
    }

    pub fn add_action(&mut self, macro_id: &str, action: MacroAction) -> bool {
        if let Some(m) = self.macros.iter_mut().find(|m| m.id == macro_id) {
            m.actions.push(action);
            true
        } else {
            false
        }
    }

    pub fn delete_macro(&mut self, id: &str) -> bool {
        if let Some(pos) = self.macros.iter().position(|m| m.id == id) {
            self.macros.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn execute_macro(&mut self, id: &str) -> Option<Vec<MacroAction>> {
        if let Some(m) = self.macros.iter_mut().find(|m| m.id == id && m.enabled) {
            m.use_count += 1;
            Some(m.actions.clone())
        } else {
            None
        }
    }

    pub fn find_by_trigger(&self, trigger: &str) -> Option<&Macro> {
        self.macros
            .iter()
            .find(|m| m.trigger == trigger && m.enabled)
    }

    pub fn get_summary(&self) -> String {
        let total = self.macros.len();
        let enabled = self.macros.iter().filter(|m| m.enabled).count();

        format!("Macros: {} | Active: {}", total, enabled)
    }
}

impl Default for MacroSystem {
    fn default() -> Self {
        Self::new()
    }
}
