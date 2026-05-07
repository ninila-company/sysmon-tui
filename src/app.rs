#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Cpu,
    Memory,
    Processes,
    Disks,
    Network,
    Docker,
}

#[allow(dead_code)]
impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Cpu => Tab::Memory,
            Tab::Memory => Tab::Processes,
            Tab::Processes => Tab::Disks,
            Tab::Disks => Tab::Network,
            Tab::Network => Tab::Docker,
            Tab::Docker => Tab::Cpu,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Cpu => "CPU",
            Tab::Memory => "Memory",
            Tab::Processes => "Processes",
            Tab::Disks => "Disks",
            Tab::Network => "Network",
            Tab::Docker => "Docker",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Pid,
    Name,
    Cpu,
    Mem,
}

#[allow(dead_code)]
impl SortColumn {
    pub fn next(&self) -> Self {
        match self {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Mem,
            SortColumn::Mem => SortColumn::Pid,
        }
    }

    pub fn header(&self) -> &str {
        match self {
            SortColumn::Pid => "PID",
            SortColumn::Name => "Name",
            SortColumn::Cpu => "CPU%",
            SortColumn::Mem => "MEM%",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub mem_percent: f32,
    pub user: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub name: String,
    pub file_system: String,
    pub total: u64,
    pub available: u64,
    pub is_removable: bool,
}

impl DiskInfo {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    pub fn usage_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.used() as f64 / self.total as f64
    }
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ipv4: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_speed: u64,
    pub tx_speed: u64,
}

#[derive(Debug, Clone)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
    pub ports: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created: String,
}

#[derive(Debug, Clone)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[allow(dead_code)]
pub struct App {
    pub running: bool,
    pub selected_tab: Tab,
    pub sort_column: SortColumn,
    pub sort_descending: bool,
     pub process_scroll: usize,
     pub process_selected: usize,
     pub visible_process_count: usize,
     pub disk_selected: usize,
     pub disk_gauges_scroll: usize,
     pub visible_disk_gauges: usize,
    pub refresh_rate_ms: u64,
    pub min_terminal_size_met: bool,
    pub show_help: bool,
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_history: Vec<f32>,
    pub cpu_cores: Vec<f32>,
    pub cpu_brand: String,
    pub cpu_freq: f32,
    pub load_average: Option<LoadAverage>,
    pub net_rx_history: Vec<u64>,
    pub net_tx_history: Vec<u64>,
    pub processes: Vec<ProcessInfo>,
    pub total_processes: usize,
    pub disks: Vec<DiskInfo>,
    pub network_interfaces: Vec<NetworkInterface>,
    pub total_network_rx: u64,
    pub total_network_tx: u64,
    pub docker_containers: Vec<DockerContainer>,
    pub docker_images: Vec<DockerImage>,
    pub docker_error: Option<String>,
    pub hostname: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_secs: u64,
}

impl App {
    pub fn new(refresh_rate_ms: u64) -> Self {
        let history_len = 60;
        Self {
            running: true,
            selected_tab: Tab::default(),
            sort_column: SortColumn::default(),
            sort_descending: true,
             process_scroll: 0,
             process_selected: 0,
             visible_process_count: 0,
             disk_selected: 0,
             disk_gauges_scroll: 0,
             visible_disk_gauges: 0,
            refresh_rate_ms,
            min_terminal_size_met: true,
            show_help: false,
            total_memory: 1,
            used_memory: 0,
            available_memory: 0,
            total_swap: 0,
            used_swap: 0,
            cpu_history: vec![0.0; history_len],
            cpu_cores: Vec::new(),
            cpu_brand: String::new(),
            cpu_freq: 0.0,
            load_average: None,
            net_rx_history: vec![0; history_len],
            net_tx_history: vec![0; history_len],
            processes: Vec::new(),
            total_processes: 0,
            disks: Vec::new(),
            network_interfaces: Vec::new(),
            total_network_rx: 0,
            total_network_tx: 0,
            docker_containers: Vec::new(),
            docker_images: Vec::new(),
            docker_error: None,
            hostname: String::new(),
            os_version: String::new(),
            kernel_version: String::new(),
            uptime_secs: 0,
        }
    }

    pub fn tick(&mut self) {
        self.cpu_history.push(0.0);
        self.cpu_history.remove(0);
        self.net_rx_history.push(0);
        self.net_rx_history.remove(0);
        self.net_tx_history.push(0);
        self.net_tx_history.remove(0);
    }

    pub fn update_cpu_history(&mut self, usage: f32) {
        if let Some(last) = self.cpu_history.last_mut() {
            *last = usage;
        }
    }

    pub fn update_memory(
        &mut self,
        total: u64,
        used: u64,
        available: u64,
        total_swap: u64,
        used_swap: u64,
    ) {
        self.total_memory = total;
        self.used_memory = used;
        self.available_memory = available;
        self.total_swap = total_swap;
        self.used_swap = used_swap;
    }

    pub fn mem_usage_percent(&self) -> f64 {
        if self.total_memory == 0 {
            return 0.0;
        }
        self.used_memory as f64 / self.total_memory as f64
    }

    pub fn swap_usage_percent(&self) -> f64 {
        if self.total_swap == 0 {
            return 0.0;
        }
        self.used_swap as f64 / self.total_swap as f64
    }

    pub fn has_swap(&self) -> bool {
        self.total_swap > 0
    }

    pub fn update_cpu_cores(&mut self, cores: Vec<f32>) {
        self.cpu_cores = cores;
    }

    pub fn update_cpu_info(&mut self, brand: String, freq: f32) {
        self.cpu_brand = brand;
        self.cpu_freq = freq;
    }

    pub fn update_load_average(&mut self, load_avg: Option<LoadAverage>) {
        self.load_average = load_avg;
    }

    pub fn update_net_history(&mut self, rx: u64, tx: u64) {
        if let Some(last_rx) = self.net_rx_history.last_mut() {
            *last_rx = rx;
        }
        if let Some(last_tx) = self.net_tx_history.last_mut() {
            *last_tx = tx;
        }
    }

    pub fn update_network(&mut self, interfaces: Vec<NetworkInterface>, rx: u64, tx: u64) {
        self.network_interfaces = interfaces;
        self.total_network_rx = rx;
        self.total_network_tx = tx;
    }

    pub fn update_docker(
        &mut self,
        containers: Vec<DockerContainer>,
        images: Vec<DockerImage>,
        error: Option<String>,
    ) {
        self.docker_containers = containers;
        self.docker_images = images;
        self.docker_error = error;
    }

    pub fn update_sys_info(&mut self, hostname: String, os: String, kernel: String, uptime: u64) {
        self.hostname = hostname;
        self.os_version = os;
        self.kernel_version = kernel;
        self.uptime_secs = uptime;
    }

    pub fn formatted_uptime(&self) -> String {
        let secs = self.uptime_secs;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        let mut parts = Vec::new();
        if days > 0 { parts.push(format!("{days}d")); }
        if hours > 0 || days > 0 { parts.push(format!("{hours:02}h")); }
        if minutes > 0 || hours > 0 || days > 0 { parts.push(format!("{minutes:02}m")); }
        parts.push(format!("{seconds:02}s"));

        parts.join(" ")
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.total_processes = processes.len();
        self.processes = processes;
        if self.process_selected >= self.processes.len() && !self.processes.is_empty() {
            self.process_selected = self.processes.len() - 1;
        } else if self.processes.is_empty() {
            self.process_selected = 0;
        }
    }

    pub fn update_disks(&mut self, disks: Vec<DiskInfo>) {
        self.disks = disks;
        if self.disk_selected >= self.disks.len() && !self.disks.is_empty() {
            self.disk_selected = self.disks.len() - 1;
        } else if self.disks.is_empty() {
            self.disk_selected = 0;
        }
    }

    pub fn sorted_processes(&self) -> Vec<&ProcessInfo> {
        let mut procs: Vec<&ProcessInfo> = self.processes.iter().collect();
        procs.sort_by(|a, b| {
            let ord = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Mem => a.memory.cmp(&b.memory),
            };
            if self.sort_descending { ord.reverse() } else { ord }
        });
        procs
    }

    pub fn scroll_up(&mut self) {
        if self.process_selected > 0 {
            self.process_selected -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.process_selected + 1 < self.processes.len() {
            self.process_selected += 1;
        }
    }

    pub fn page_up(&mut self) {
        let step = self.visible_process_count.max(10);
        self.process_selected = self.process_selected.saturating_sub(step);
    }

    pub fn page_down(&mut self) {
        let step = self.visible_process_count.max(10);
        self.process_selected += step;
        if self.process_selected >= self.processes.len() && !self.processes.is_empty() {
            self.process_selected = self.processes.len() - 1;
        }
    }

    pub fn disk_scroll_up(&mut self) {
        if self.disk_selected > 0 {
            self.disk_selected -= 1;
        }
    }

    pub fn disk_scroll_down(&mut self) {
        if self.disk_selected + 1 < self.disks.len() {
            self.disk_selected += 1;
        }
    }

    pub fn disk_page_up(&mut self) {
        let step = 5; // approximate visible disk count in table
        self.disk_selected = self.disk_selected.saturating_sub(step);
    }

    pub fn disk_page_down(&mut self) {
        let step = 5;
        self.disk_selected += step;
        if self.disk_selected >= self.disks.len() && !self.disks.is_empty() {
            self.disk_selected = self.disks.len() - 1;
        }
    }

    pub fn disk_gauges_scroll_up(&mut self) {
        self.disk_gauges_scroll = self.disk_gauges_scroll.saturating_sub(1);
    }

    pub fn disk_gauges_scroll_down(&mut self, visible: usize) {
        if visible == 0 { return; }
        let max_scroll = self.disks.len().saturating_sub(visible);
        if self.disk_gauges_scroll < max_scroll {
            self.disk_gauges_scroll += 1;
        }
    }
}
