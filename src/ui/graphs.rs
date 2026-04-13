use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub values: VecDeque<f32>,
    pub max_points: usize,
    pub min_val: f32,
    pub max_val: f32,
    pub avg_val: f32,
    pub peak_val: f32,
}

impl GraphData {
    pub fn new(max_points: usize) -> Self {
        Self {
            values: VecDeque::new(),
            max_points,
            min_val: 0.0,
            max_val: 100.0,
            avg_val: 0.0,
            peak_val: 0.0,
        }
    }

    pub fn add(&mut self, value: f32) {
        self.values.push_front(value);
        if self.values.len() > self.max_points {
            self.values.pop_back();
        }

        if value > self.max_val {
            self.max_val = value;
        }
        if value < self.min_val || self.min_val == 0.0 {
            self.min_val = value;
        }
        if value > self.peak_val {
            self.peak_val = value;
        }

        self.avg_val = self.values.iter().sum::<f32>() / self.values.len() as f32;
    }

    pub fn get_normalized(&self) -> Vec<f32> {
        let range = self.max_val - self.min_val;
        if range == 0.0 {
            return self.values.iter().map(|_| 0.5).collect();
        }
        self.values
            .iter()
            .map(|v| (v - self.min_val) / range)
            .collect()
    }

    pub fn get_sparkline(&self, width: usize) -> String {
        let data = self.get_normalized();
        if data.is_empty() {
            return "─".repeat(width);
        }

        let chars = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '█'];
        let step = (data.len() as f32 / width as f32).max(1.0);

        let mut line = String::new();
        for i in 0..width {
            let idx = (i as f32 * step) as usize;
            if idx < data.len() {
                let val = data[idx];
                let char_idx =
                    ((val * (chars.len() - 1) as f32).round() as usize).min(chars.len() - 1);
                line.push(chars[char_idx]);
            } else {
                line.push('─');
            }
        }
        line
    }
}

pub struct RealtimeGraph {
    pub cpu: GraphData,
    pub memory: GraphData,
    pub network: GraphData,
    pub power: GraphData,
    pub temperature: GraphData,
    pub processing: GraphData,
    pub disk: GraphData,
    pub gpu: GraphData,
    pub frame_count: u64,
}

impl RealtimeGraph {
    pub fn new() -> Self {
        Self {
            cpu: GraphData::new(60),
            memory: GraphData::new(60),
            network: GraphData::new(60),
            power: GraphData::new(60),
            temperature: GraphData::new(60),
            processing: GraphData::new(60),
            disk: GraphData::new(60),
            gpu: GraphData::new(60),
            frame_count: 0,
        }
    }

    pub fn update(
        &mut self,
        cpu: f32,
        memory: f32,
        network: f32,
        power: f32,
        temp: f32,
        load: f32,
    ) {
        self.frame_count += 1;

        self.cpu.add(cpu);
        self.memory.add(memory);
        self.network.add(network);
        self.power.add(power);
        self.temperature.add(temp);
        self.processing.add(load);

        if self.frame_count % 10 == 0 {
            self.disk.add(rand_float(20.0, 60.0));
            self.gpu.add(rand_float(30.0, 80.0));
        }
    }
}

impl Default for RealtimeGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_float(min: f32, max: f32) -> f32 {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    min + (seed as f32 / u32::MAX as f32) * (max - min)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "INFO"),
            AlertLevel::Warning => write!(f, "WARNING"),
            AlertLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemAlert {
    pub id: usize,
    pub message: String,
    pub level: AlertLevel,
    pub timestamp: u64,
    pub source: String,
}

impl SystemAlert {
    pub fn new(message: String, level: AlertLevel, source: &str) -> Self {
        Self {
            id: rand_id(),
            message,
            level,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source: source.to_string(),
        }
    }
}

fn rand_id() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize
}

pub struct AlertManager {
    pub alerts: Vec<SystemAlert>,
    pub max_alerts: usize,
    pub alert_history: Vec<SystemAlert>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Vec::new(),
            max_alerts: 10,
            alert_history: Vec::new(),
        }
    }

    pub fn add(&mut self, message: String, level: AlertLevel, source: &str) {
        let alert = SystemAlert::new(message, level, source);
        self.alerts.insert(0, alert.clone());
        self.alert_history.push(alert);

        if self.alerts.len() > self.max_alerts {
            self.alerts.pop();
        }

        if self.alert_history.len() > 100 {
            self.alert_history.remove(0);
        }
    }

    pub fn get_critical(&self) -> Vec<&SystemAlert> {
        self.alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Critical)
            .collect()
    }

    pub fn get_warning(&self) -> Vec<&SystemAlert> {
        self.alerts
            .iter()
            .filter(|a| a.level == AlertLevel::Warning)
            .collect()
    }

    pub fn clear(&mut self) {
        self.alerts.clear();
    }

    pub fn get_history_count(&self) -> usize {
        self.alert_history.len()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct HexagonMonitor {
    pub segments: Vec<f32>,
    pub labels: Vec<String>,
    pub max_value: f32,
}

impl HexagonMonitor {
    pub fn new() -> Self {
        Self {
            segments: vec![50.0; 6],
            labels: vec![
                "CPU".to_string(),
                "MEM".to_string(),
                "NET".to_string(),
                "GPU".to_string(),
                "DISK".to_string(),
                "TEMP".to_string(),
            ],
            max_value: 100.0,
        }
    }

    pub fn update(&mut self, values: [f32; 6]) {
        for (i, v) in values.iter().enumerate() {
            if i < self.segments.len() {
                self.segments[i] = v.min(self.max_value);
            }
        }
    }

    pub fn set_labels(&mut self, labels: Vec<String>) {
        if labels.len() == 6 {
            self.labels = labels;
        }
    }
}

impl Default for HexagonMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GaugeWidget {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub label: String,
    pub unit: String,
    pub warning_threshold: f32,
    pub critical_threshold: f32,
}

impl GaugeWidget {
    pub fn new(label: &str, unit: &str, min: f32, max: f32) -> Self {
        Self {
            value: min,
            min,
            max,
            label: label.to_string(),
            unit: unit.to_string(),
            warning_threshold: max * 0.7,
            critical_threshold: max * 0.9,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        if self.value >= self.critical_threshold {
            (255, 0, 0)
        } else if self.value >= self.warning_threshold {
            (255, 200, 0)
        } else {
            (0, 255, 200)
        }
    }

    pub fn get_percentage(&self) -> f32 {
        ((self.value - self.min) / (self.max - self.min)) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct StatusIndicator {
    pub name: String,
    pub status: StatusLevel,
    pub message: String,
    pub last_update: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusLevel {
    Online,
    Warning,
    Error,
    Offline,
}

impl StatusIndicator {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: StatusLevel::Offline,
            message: String::new(),
            last_update: 0,
        }
    }

    pub fn set_status(&mut self, status: StatusLevel, message: &str) {
        self.status = status;
        self.message = message.to_string();
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}
