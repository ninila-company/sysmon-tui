use anyhow::Result;
use clap::Parser;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, stdout};
use tokio::sync::mpsc;

mod app;
mod events;
mod ui;

use app::App;
use events::{handle_event, apply_action};

#[derive(Parser, Debug)]
#[command(name = "sysmon-tui", version, about = "System monitor TUI")]
struct Args {
    #[arg(short, long, default_value_t = 1000)]
    refresh_rate: u64,
}

#[allow(dead_code)]
struct SystemMetrics {
    cpu_usage: f32,
    cpu_cores: Vec<f32>,
    cpu_brand: String,
    cpu_freq: f32,
    load_average: Option<app::LoadAverage>,
    total_memory: u64,
    used_memory: u64,
    available_memory: u64,
    total_swap: u64,
    used_swap: u64,
    processes: Vec<app::ProcessInfo>,
    disks: Vec<app::DiskInfo>,
    net_rx: u64,
    net_tx: u64,
    network_interfaces: Vec<app::NetworkInterface>,
    docker_containers: Vec<app::DockerContainer>,
    docker_images: Vec<app::DockerImage>,
    docker_error: Option<String>,
    hostname: String,
    os_version: String,
    kernel_version: String,
    uptime_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (tx, mut rx) = mpsc::channel::<SystemMetrics>(32);

    let refresh_rate = args.refresh_rate;
    tokio::spawn(async move {
        collector_task(tx, refresh_rate).await;
    });

    let mut terminal = setup_terminal()?;

    let mut app = App::new(args.refresh_rate);

    while app.running {
        if let Some(metrics) = rx.try_recv().ok() {
            app.update_memory(
                metrics.total_memory,
                metrics.used_memory,
                metrics.available_memory,
                metrics.total_swap,
                metrics.used_swap,
            );
            app.update_cpu_history(metrics.cpu_usage);
            app.update_cpu_cores(metrics.cpu_cores);
            app.update_cpu_info(metrics.cpu_brand, metrics.cpu_freq);
            app.update_load_average(metrics.load_average);
            app.update_net_history(metrics.net_rx, metrics.net_tx);
            app.update_processes(metrics.processes);
            app.update_disks(metrics.disks);
            app.update_network(metrics.network_interfaces, metrics.net_rx, metrics.net_tx);
            app.update_docker(metrics.docker_containers, metrics.docker_images, metrics.docker_error);
            app.update_sys_info(metrics.hostname, metrics.os_version, metrics.kernel_version, metrics.uptime_secs);
            app.tick();
        }

        terminal.draw(|frame| ui::draw(frame, &app))?;

        let action = handle_event(&mut app)?;
        apply_action(&mut app, action);
    }

    restore_terminal()?;
    Ok(())
}

async fn collector_task(tx: mpsc::Sender<SystemMetrics>, interval_ms: u64) {
    use sysinfo::{Disks, Networks, System};

    let mut sys = System::new_all();
    let mut sys_disks = Disks::new_with_refreshed_list();
    let mut prev_net_rx: u64 = 0;
    let mut prev_net_tx: u64 = 0;
    let mut first = true;

    sys.refresh_cpu_usage();
    let cpu_brand = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_freq = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);

    loop {
        sys.refresh_all();
        sys_disks.refresh(true);
        let networks = Networks::new_with_refreshed_list();

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;
        for (_iface, data) in &networks {
            total_rx += data.received();
            total_tx += data.transmitted();
        }

        let network_interfaces: Vec<app::NetworkInterface> = networks
            .iter()
            .map(|(name, data)| {
                let mac = data.mac_address().to_string();
                let ipv4 = data.ip_networks().iter()
                    .find(|ip| ip.addr.is_ipv4())
                    .map(|ip| ip.addr.to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                app::NetworkInterface {
                    name: name.clone(),
                    mac_address: mac,
                    ipv4,
                    rx_bytes: data.total_received(),
                    tx_bytes: data.total_transmitted(),
                    rx_speed: data.received(),
                    tx_speed: data.transmitted(),
                }
            })
            .collect();

        let cpu_usage = sys.global_cpu_usage();
        let cpu_cores: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();

        let processes: Vec<app::ProcessInfo> = sys
            .processes()
            .values()
            .map(|p| {
                let uid = p.user_id().map(|u| u.to_string()).unwrap_or_else(|| "-".to_string());
                let cmd = p.cmd().first()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                app::ProcessInfo {
                    pid: p.pid().as_u32(),
                    name: p.name().to_string_lossy().to_string(),
                    cpu_usage: p.cpu_usage(),
                    memory: p.memory(),
                    mem_percent: p.memory() as f32 / sys.total_memory() as f32 * 100.0,
                    user: uid,
                    command: cmd,
                }
            })
            .collect();

        let disks: Vec<app::DiskInfo> = sys_disks
            .iter()
            .map(|d| {
                app::DiskInfo {
                    mount_point: d.mount_point().to_string_lossy().to_string(),
                    name: d.name().to_string_lossy().to_string(),
                    file_system: d.file_system().to_string_lossy().to_string(),
                    total: d.total_space(),
                    available: d.available_space(),
                    is_removable: d.is_removable(),
                }
            })
            .collect();

        let (docker_containers, docker_images, docker_error) = fetch_docker_data().await;

        let la = sysinfo::System::load_average();
        let load_average = Some(app::LoadAverage {
            one: la.one,
            five: la.five,
            fifteen: la.fifteen,
        });

        let rx_delta = if first {
            first = false;
            0
        } else {
            total_rx.saturating_sub(prev_net_rx)
        };
        let tx_delta = if first { 0 } else { total_tx.saturating_sub(prev_net_tx) };

        prev_net_rx = total_rx;
        prev_net_tx = total_tx;

        let metrics = SystemMetrics {
            cpu_usage,
            cpu_cores,
            cpu_brand: cpu_brand.clone(),
            cpu_freq: cpu_freq as f32,
            load_average,
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            available_memory: sys.available_memory(),
            total_swap: sys.total_swap(),
            used_swap: sys.used_swap(),
            processes,
            disks,
            network_interfaces,
            net_rx: rx_delta,
            net_tx: tx_delta,
            docker_containers,
            docker_images,
            docker_error,
            hostname: System::host_name().unwrap_or_else(|| "N/A".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "N/A".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
            uptime_secs: sysinfo::System::uptime(),
        };

        if tx.send(metrics).await.is_err() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
    }
}

async fn fetch_docker_data() -> (
    Vec<app::DockerContainer>,
    Vec<app::DockerImage>,
    Option<String>,
) {
    use docker_api::Docker;
    use docker_api::opts::ContainerListOpts;

    let docker = Docker::unix("/var/run/docker.sock");

    let containers = match docker.containers().list(&ContainerListOpts::builder().all(true).build()).await {
        Ok(list) => list
            .iter()
            .map(|c| {
                let name = c.names.as_ref()
                    .and_then(|n: &Vec<String>| n.first())
                    .map(|s: &String| s.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                let image = c.image.clone().unwrap_or_else(|| "N/A".to_string());
                let state = c.state.clone().unwrap_or_default();
                let status = c.status.clone().unwrap_or_default();
                let created = c.created.map(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "N/A".to_string())
                }).unwrap_or_else(|| "N/A".to_string());

                let id = c.id.as_ref()
                    .map(|s: &String| s.chars().take(12).collect())
                    .unwrap_or_else(|| "N/A".to_string());

                let ports: Vec<String> = c.ports.as_ref().map(|pl: &Vec<docker_api::models::Port>| {
                    pl.iter().filter_map(|p| {
                        if let (Some(ip), Some(pp)) = (&p.ip, p.public_port) {
                            Some(format!("{ip}:{pp}->{ppr}", ppr = p.private_port))
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>()
                }).unwrap_or_default();

                app::DockerContainer {
                    id,
                    name,
                    image,
                    status,
                    created,
                    ports: ports.join(", "),
                    state,
                }
            })
            .collect(),
        Err(_e) => vec![],
    };

    let images = match docker.images().list(&Default::default()).await {
        Ok(list) => list
            .iter()
            .map(|img| {
                let repo_tags: Vec<String> = img.repo_tags.iter()
                    .filter(|t| !(*t).starts_with("<none>"))
                    .cloned()
                    .collect();

                let (repository, tag) = if let Some(t) = repo_tags.first() {
                    if let Some(idx) = t.rfind(':') {
                        (t[..idx].to_string(), t[idx + 1..].to_string())
                    } else {
                        (t.clone(), "latest".to_string())
                    }
                } else {
                    ("<none>".to_string(), "<none>".to_string())
                };

                let id = img.id.replace("sha256:", "").chars().take(12).collect::<String>();

                let size = format_bytes(img.size.unsigned_abs() as u64);
                let created = chrono::DateTime::from_timestamp(img.created as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                app::DockerImage {
                    id,
                    repository,
                    tag,
                    size,
                    created,
                }
            })
            .collect(),
        Err(_) => vec![],
    };

    (containers, images, None)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.2} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
