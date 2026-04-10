use std::collections::VecDeque;

pub struct CommandInput {
    pub current: String,
    pub history: VecDeque<String>,
    pub history_index: Option<usize>,
    pub suggestions: Vec<String>,
    pub selected_suggestion: Option<usize>,
    pub max_history: usize,
}

impl Default for CommandInput {
    fn default() -> Self {
        Self {
            current: String::new(),
            history: VecDeque::new(),
            history_index: None,
            suggestions: Vec::new(),
            selected_suggestion: None,
            max_history: 100,
        }
    }
}

impl CommandInput {
    pub fn push_command(&mut self, command: String) {
        if !command.is_empty() {
            self.history.push_front(command.clone());
            if self.history.len() > self.max_history {
                self.history.pop_back();
            }
        }
        self.current.clear();
        self.history_index = None;
        self.suggestions.clear();
        self.selected_suggestion = None;
    }

    pub fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_index = match self.history_index {
            None => 0,
            Some(i) => {
                if i < self.history.len() - 1 {
                    i + 1
                } else {
                    return;
                }
            }
        };
        self.history_index = Some(new_index);
        if let Some(cmd) = self.history.get(new_index) {
            self.current = cmd.clone();
        }
    }

    pub fn navigate_history_down(&mut self) {
        match self.history_index {
            None => return,
            Some(0) => {
                self.history_index = None;
                self.current.clear();
            }
            Some(i) => {
                let new_index = i - 1;
                self.history_index = Some(new_index);
                if let Some(cmd) = self.history.get(new_index) {
                    self.current = cmd.clone();
                }
            }
        }
    }

    pub fn update_suggestions(&mut self) {
        if self.current.is_empty() {
            self.suggestions.clear();
            self.selected_suggestion = None;
            return;
        }

        let commands = vec![
            "help", "status", "clear", "ai ", "run ", "open ", "close ",
            "system info", "system stats", "network scan", "files list",
            "settings", "execute ", "search ", "ping ", "curl ",
        ];

        self.suggestions = commands
            .iter()
            .filter(|c| c.starts_with(&self.current.to_lowercase()))
            .map(|c| c.to_string())
            .collect();
        
        if !self.suggestions.is_empty() {
            self.selected_suggestion = Some(0);
        }
    }

    pub fn select_next_suggestion(&mut self) {
        if let Some(idx) = self.selected_suggestion {
            if idx < self.suggestions.len() - 1 {
                self.selected_suggestion = Some(idx + 1);
            }
        }
    }

    pub fn select_prev_suggestion(&mut self) {
        if let Some(idx) = self.selected_suggestion {
            if idx > 0 {
                self.selected_suggestion = Some(idx - 1);
            }
        }
    }

    pub fn apply_selected_suggestion(&mut self) {
        if let Some(idx) = self.selected_suggestion {
            if let Some(suggestion) = self.suggestions.get(idx) {
                self.current = suggestion.clone();
                self.selected_suggestion = None;
                self.suggestions.clear();
            }
        }
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.history.clear();
        self.history_index = None;
        self.suggestions.clear();
        self.selected_suggestion = None;
    }
}