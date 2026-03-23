use std::fs;
use std::net::UdpSocket;
use std::process::Command;

#[derive(Default)]
pub struct CpuUsageSampler {
    prev_total: Option<u64>,
    prev_idle: Option<u64>,
    smoothed_usage: f32,
}

pub fn read_distro_name() -> String {
    let content = match fs::read_to_string("/etc/os-release") {
        Ok(content) => content,
        Err(_) => return std::env::consts::OS.to_string(),
    };

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return value.trim_matches('"').to_string();
        }
    }

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            return value.trim_matches('"').to_string();
        }
    }

    std::env::consts::OS.to_string()
}

pub fn read_host_uptime_secs() -> Option<u64> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let first = content.split_whitespace().next()?;
    let secs = first.parse::<f64>().ok()?;
    if secs.is_sign_negative() {
        return None;
    }
    Some(secs as u64)
}

pub fn read_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn read_kernel_release() -> String {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_cpu_totals() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    let mut parts = line.split_whitespace();
    if parts.next()? != "cpu" {
        return None;
    }

    let user = parts.next()?.parse::<u64>().ok()?;
    let nice = parts.next()?.parse::<u64>().ok()?;
    let system = parts.next()?.parse::<u64>().ok()?;
    let idle = parts.next()?.parse::<u64>().ok()?;
    let iowait = parts
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let irq = parts
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let softirq = parts
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let steal = parts
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let idle_all = idle + iowait;
    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    Some((total, idle_all))
}

pub fn read_cpu_usage_sample(sampler: &mut CpuUsageSampler) -> Option<f32> {
    let (total, idle) = read_cpu_totals()?;
    let (prev_total, prev_idle) = match (sampler.prev_total, sampler.prev_idle) {
        (Some(t), Some(i)) => (t, i),
        _ => {
            sampler.prev_total = Some(total);
            sampler.prev_idle = Some(idle);
            return None;
        }
    };

    sampler.prev_total = Some(total);
    sampler.prev_idle = Some(idle);

    let delta_total = total.saturating_sub(prev_total);
    let delta_idle = idle.saturating_sub(prev_idle);
    if delta_total == 0 {
        return Some(sampler.smoothed_usage);
    }

    let raw_usage = ((delta_total.saturating_sub(delta_idle)) as f32 / delta_total as f32).clamp(0.0, 1.0);
    sampler.smoothed_usage = sampler.smoothed_usage * 0.65 + raw_usage * 0.35;
    Some(sampler.smoothed_usage)
}

pub fn read_memory_usage() -> Option<String> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kib: Option<u64> = None;
    let mut available_kib: Option<u64> = None;

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total_kib = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok());
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available_kib = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok());
        }

        if total_kib.is_some() && available_kib.is_some() {
            break;
        }
    }

    let total = total_kib?;
    let available = available_kib?;
    if total == 0 || available > total {
        return None;
    }

    let used_pct = ((total - available) as f64 / total as f64) * 100.0;
    Some(format!("{:.1}%", used_pct))
}

pub fn read_username() -> String {
    std::env::var("USER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn read_shell_name() -> String {
    let shell = std::env::var("SHELL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    shell.rsplit('/').next().unwrap_or("unknown").to_string()
}

fn read_default_iface() -> Option<String> {
    let content = fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() > 2 && cols[1] == "00000000" {
            return Some(cols[0].to_string());
        }
    }
    None
}

fn read_local_ipv4() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

pub fn read_network_summary() -> String {
    let iface = read_default_iface().unwrap_or_else(|| "-".to_string());
    let ip = read_local_ipv4().unwrap_or_else(|| "n/a".to_string());
    format!("{} {}", iface, ip)
}

pub fn read_process_rss() -> Option<String> {
    let content = fs::read_to_string("/proc/self/status").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let kib = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok())?;
            let mib = kib as f64 / 1024.0;
            return Some(format!("{:.1} MiB", mib));
        }
    }
    None
}

pub fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;
    let seconds = total_secs % 60;

    format!("{:02}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
}

fn parse_first_hz_token(text: &str) -> Option<u64> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| c == '*' || c == '+' || c == '@');
        if let Ok(v) = cleaned.parse::<f64>() {
            if (20.0..=360.0).contains(&v) {
                return Some(v.round() as u64);
            }
        }
    }
    None
}

fn detect_refresh_from_xrandr() -> Option<u64> {
    let out = Command::new("xrandr").arg("--current").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.contains('*') {
            if let Some(hz) = parse_first_hz_token(line) {
                return Some(hz);
            }
        }
    }
    None
}

fn detect_refresh_from_wlr_randr() -> Option<u64> {
    let out = Command::new("wlr-randr").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.contains(" Hz") {
            if let Some(hz) = parse_first_hz_token(line) {
                return Some(hz);
            }
        }
    }
    None
}

fn detect_refresh_from_hyprctl() -> Option<u64> {
    let out = Command::new("hyprctl").arg("monitors").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.contains("refreshRate") {
            if let Some(hz) = parse_first_hz_token(line) {
                return Some(hz);
            }
        }
    }
    None
}

pub fn detect_display_refresh_hz() -> Option<u64> {
    detect_refresh_from_xrandr()
        .or_else(detect_refresh_from_wlr_randr)
        .or_else(detect_refresh_from_hyprctl)
}
