use anyhow::Result;

const GPU_THROTTLE_TEMP_C: f32 = 85.0;
const CPU_THROTTLE_TEMP_C: f32 = 85.0;
const THROTTLE_CONTEXT_SIZE: usize = 2048;
const LOW_VRAM_THRESHOLD_MB: u64 = 4096;
const MEDIUM_VRAM_THRESHOLD_MB: u64 = 8192;
const RAM_OVERHEAD_MB: u64 = 2048;
const GPU_VRAM_HIGH_MB: u64 = 16_000;
const GPU_VRAM_MEDIUM_MB: u64 = 8_000;
const GPU_VRAM_LOW_MB: u64 = 4_000;

#[derive(Debug, Clone)]
pub struct HardwareMonitor {
    pub gpu_temp_c: f32,
    pub cpu_temp_c: f32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub is_throttling: bool,
}

impl HardwareMonitor {
    pub fn new() -> Self {
        Self {
            gpu_temp_c: 0.0,
            cpu_temp_c: 0.0,
            vram_used_mb: 0,
            vram_total_mb: 0,
            ram_used_mb: 0,
            ram_total_mb: 0,
            is_throttling: false,
        }
    }

    pub async fn sample() -> Self {
        let gpu_temp = Self::gpu_temp().unwrap_or(0.0);
        let cpu_temp = Self::cpu_temp().unwrap_or(0.0);
        let (vram_used, vram_total) = Self::vram_status().unwrap_or((0, 0));
        let (ram_used, ram_total) = Self::ram_status();
        let throttling = gpu_temp > GPU_THROTTLE_TEMP_C || cpu_temp > CPU_THROTTLE_TEMP_C;

        Self {
            gpu_temp_c: gpu_temp,
            cpu_temp_c: cpu_temp,
            vram_used_mb: vram_used,
            vram_total_mb: vram_total,
            ram_used_mb: ram_used,
            ram_total_mb: ram_total,
            is_throttling: throttling,
        }
    }

    pub fn safe_to_load(&self, needed_vram_mb: u64) -> bool {
        if self.is_throttling {
            return false;
        }
        if self.vram_total_mb > 0 {
            let available = self.vram_total_mb.saturating_sub(self.vram_used_mb);
            if available < needed_vram_mb {
                return false;
            }
        }
        if self.vram_total_mb == 0 {
            if self.ram_total_mb > 0 {
                let available_ram = self.ram_total_mb.saturating_sub(self.ram_used_mb);
                if available_ram < needed_vram_mb + RAM_OVERHEAD_MB {
                    return false;
                }
            }
        }
        true
    }

    pub fn safe_context_size(&self, desired: usize) -> usize {
        if self.is_throttling {
            return THROTTLE_CONTEXT_SIZE;
        }
        if self.vram_total_mb > 0 {
            let available = self.vram_total_mb.saturating_sub(self.vram_used_mb);
            if available < LOW_VRAM_THRESHOLD_MB {
                return THROTTLE_CONTEXT_SIZE;
            }
            if available < MEDIUM_VRAM_THRESHOLD_MB {
                return (LOW_VRAM_THRESHOLD_MB as usize).min(desired);
            }
        }
        desired
    }

    pub fn recommended_quant(&self) -> &'static str {
        if self.vram_total_mb >= GPU_VRAM_HIGH_MB { "Q8_0" }
        else if self.vram_total_mb >= GPU_VRAM_MEDIUM_MB { "Q5_K_M" }
        else if self.vram_total_mb >= GPU_VRAM_LOW_MB { "Q4_K_M" }
        else { "Q4_K_M" }
    }

    fn gpu_temp() -> Result<f32> {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
            .output()
        {
            let temp_string = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(temp) = temp_string.parse::<f32>() {
                return Ok(temp);
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(["--showtemp"])
            .output()
        {
            let temp_string = String::from_utf8_lossy(&output.stdout);
            for line in temp_string.lines() {
                if let Some(temp_str) = line.split_whitespace()
                    .find(|word| word.ends_with('C') && word.len() > 1)
                {
                    if let Ok(temp) = temp_str.trim_end_matches('C').parse::<f32>() {
                        return Ok(temp);
                    }
                }
            }
        }
        Ok(0.0)
    }

    fn cpu_temp() -> Result<f32> {
        for thermal_path in &[
            "/sys/class/thermal/thermal_zone0/temp",
            "/sys/class/hwmon/hwmon0/temp1_input",
        ] {
            if let Ok(content) = std::fs::read_to_string(thermal_path) {
                if let Ok(millidegrees) = content.trim().parse::<f32>() {
                    return Ok(millidegrees / 1000.0);
                }
            }
        }
        Ok(0.0)
    }

    fn vram_status() -> Result<(u64, u64)> {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
            .output()
        {
            let output_string = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = output_string.trim().split(',').collect();
            if parts.len() == 2 {
                let used = parts[0].trim().parse::<u64>().unwrap_or(0);
                let total = parts[1].trim().parse::<u64>().unwrap_or(0);
                return Ok((used, total));
            }
        }
        Ok((0, 0))
    }

    fn ram_status() -> (u64, u64) {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut available = 0u64;
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        total = kb.parse::<u64>().unwrap_or(0) / 1024;
                    }
                }
                if line.starts_with("MemAvailable:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        available = kb.parse::<u64>().unwrap_or(0) / 1024;
                    }
                }
            }
            return (total.saturating_sub(available), total);
        }
        (0, 8192)
    }
}
