use sysinfo::System;

pub struct SystemStats {
    system: System,
}

impl Default for SystemStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemStats {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    pub fn memory_used_mb(&self) -> u64 {
        self.system.used_memory() / 1024 / 1024
    }

    pub fn memory_total_mb(&self) -> u64 {
        self.system.total_memory() / 1024 / 1024
    }

    pub fn memory_percent(&self) -> f32 {
        let total = self.system.total_memory() as f32;
        let used = self.system.used_memory() as f32;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }

    pub fn hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn os_name(&self) -> String {
        System::name().unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn cpu_count(&self) -> usize {
        self.system.cpus().len()
    }

    pub fn summary(&self) -> String {
        format!(
            "CPU: {:.1}% | RAM: {}MB / {}MB ({:.1}%) | CPUs: {}",
            self.cpu_usage(),
            self.memory_used_mb(),
            self.memory_total_mb(),
            self.memory_percent(),
            self.cpu_count()
        )
    }
}