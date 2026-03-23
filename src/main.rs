use std::io::{self, Stdout, Write};
use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use rand::Rng;
use rand::seq::SliceRandom;

mod commands;
mod preferences;
mod patterns;
mod system_metrics;

use commands::{build_prompt_line, complete_command_input};
use patterns::{CharStyle, ColorTheme, ProgramMode, Stream, color_band_for_char, load_graph_chars, random_stream, render_scene, update_rain_streams};
use preferences::{Preferences, load_preferences, save_preferences};
use system_metrics::{
    CpuUsageSampler, detect_display_refresh_hz, format_uptime, read_cpu_usage_sample, read_distro_name, read_host_uptime_secs,
    read_hostname, read_kernel_release, read_memory_usage, read_network_summary, read_process_rss,
    read_shell_name, read_username,
};

const DEFAULT_FPS: u64 = 60;
const BASE_ANIM_FPS: f32 = 30.0;
const CURSOR_HOME: &str = "\x1b[H";
const SYNC_UPDATE_BEGIN: &str = "\x1b[?2026h";
const SYNC_UPDATE_END: &str = "\x1b[?2026l";
const COLOR_GRAY: &str = "\x1b[90m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_RESET: &str = "\x1b[39m";
const COLOR_INFO_BAR: &str = "\x1b[38;5;255m";
const COLOR_FLASH_ORANGE: &str = "\x1b[38;5;208m";
const COLOR_FLASH_YELLOW: &str = "\x1b[38;5;226m";
const COLOR_FLASH_RED: &str = "\x1b[38;5;196m";
const INFO_ROWS: usize = 3;
const INFO_SWITCH_MIN_SECS: u64 = 4;
const INFO_SWITCH_MAX_SECS: u64 = 20;
const INFO_FLASH_OUT_MIN_FRAMES: u8 = 4;
const INFO_FLASH_OUT_MAX_FRAMES: u8 = 10;
const INFO_FLASH_IN_MIN_FRAMES: u8 = 4;
const INFO_FLASH_IN_MAX_FRAMES: u8 = 8;
const INFO_FLASH_TOGGLE_EVERY_FRAMES: u8 = 1;
const LOAD_GRAPH_WIDTH: usize = 14;

struct RenderBuffers {
    front_screen: Vec<char>,
    back_screen: Vec<char>,
    output: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InfoField {
    Os,
    Host,
    Kernel,
    User,
    Shell,
    Network,
    CpuCores,
    LoadAvg,
    Memory,
    Pid,
    ProcMem,
    SystemUptime,
    AppUptime,
    Style,
    Fps,
    Terminal,
    Columns,
    Arch,
}

const INFO_FIELDS: [InfoField; 18] = [
    InfoField::Os,
    InfoField::Host,
    InfoField::Kernel,
    InfoField::User,
    InfoField::Shell,
    InfoField::Network,
    InfoField::CpuCores,
    InfoField::LoadAvg,
    InfoField::Memory,
    InfoField::Pid,
    InfoField::ProcMem,
    InfoField::SystemUptime,
    InfoField::AppUptime,
    InfoField::Style,
    InfoField::Fps,
    InfoField::Terminal,
    InfoField::Columns,
    InfoField::Arch,
];

#[derive(Clone, Copy)]
struct InfoSlot {
    field: InfoField,
    next_switch_at: u64,
    pending_field: Option<InfoField>,
    transition: InfoTransition,
    transition_frames_left: u8,
    blink_counter: u8,
    flash_color_phase: u8,
    is_visible: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InfoTransition {
    Stable,
    FlashOut,
    FlashIn,
}

struct InfoSnapshot<'a> {
    distro: &'a str,
    hostname: &'a str,
    kernel_release: &'a str,
    username: &'a str,
    shell: &'a str,
    network: &'a str,
    host_uptime: &'a str,
    app_uptime: &'a str,
    load_graph: &'a str,
    memory_usage: &'a str,
    pid: u32,
    process_memory: &'a str,
    style: CharStyle,
    fps: u64,
    width: u16,
    height: u16,
    columns: usize,
    cpu_cores: usize,
}


impl InfoField {
    fn label(self) -> &'static str {
        match self {
            InfoField::Os => "OS",
            InfoField::Host => "Host",
            InfoField::Kernel => "Kernel",
            InfoField::User => "User",
            InfoField::Shell => "Shell",
            InfoField::Network => "Net",
            InfoField::CpuCores => "Cores",
            InfoField::LoadAvg => "Load",
            InfoField::Memory => "Mem",
            InfoField::Pid => "PID",
            InfoField::ProcMem => "RSS",
            InfoField::SystemUptime => "System",
            InfoField::AppUptime => "App",
            InfoField::Style => "Style",
            InfoField::Fps => "FPS",
            InfoField::Terminal => "Terminal",
            InfoField::Columns => "Columns",
            InfoField::Arch => "Arch",
        }
    }

    fn value(self, snapshot: &InfoSnapshot<'_>) -> String {
        match self {
            InfoField::Os => snapshot.distro.to_string(),
            InfoField::Host => snapshot.hostname.to_string(),
            InfoField::Kernel => snapshot.kernel_release.to_string(),
            InfoField::User => snapshot.username.to_string(),
            InfoField::Shell => snapshot.shell.to_string(),
            InfoField::Network => snapshot.network.to_string(),
            InfoField::CpuCores => snapshot.cpu_cores.to_string(),
            InfoField::LoadAvg => snapshot.load_graph.to_string(),
            InfoField::Memory => snapshot.memory_usage.to_string(),
            InfoField::Pid => snapshot.pid.to_string(),
            InfoField::ProcMem => snapshot.process_memory.to_string(),
            InfoField::SystemUptime => snapshot.host_uptime.to_string(),
            InfoField::AppUptime => snapshot.app_uptime.to_string(),
            InfoField::Style => snapshot.style.as_str().to_string(),
            InfoField::Fps => snapshot.fps.to_string(),
            InfoField::Terminal => format!("{}x{}", snapshot.width, snapshot.height),
            InfoField::Columns => snapshot.columns.to_string(),
            InfoField::Arch => std::env::consts::ARCH.to_string(),
        }
    }
}

fn schedule_next_info_switch(now_secs: u64, rng: &mut impl Rng) -> u64 {
    now_secs + rng.random_range(INFO_SWITCH_MIN_SECS..=INFO_SWITCH_MAX_SECS)
}

fn random_flash_out_frames(rng: &mut impl Rng) -> u8 {
    rng.random_range(INFO_FLASH_OUT_MIN_FRAMES..=INFO_FLASH_OUT_MAX_FRAMES)
}

fn random_flash_in_frames(rng: &mut impl Rng) -> u8 {
    rng.random_range(INFO_FLASH_IN_MIN_FRAMES..=INFO_FLASH_IN_MAX_FRAMES)
}

fn pick_unique_next_info_field(
    slot_index: usize,
    slots: &[InfoSlot; INFO_ROWS * 2],
    rng: &mut impl Rng,
) -> Option<InfoField> {
    let mut candidates: Vec<InfoField> = INFO_FIELDS
        .iter()
        .copied()
        .filter(|candidate| {
            if *candidate == slots[slot_index].field {
                return false;
            }

            for (idx, slot) in slots.iter().enumerate() {
                if idx == slot_index {
                    continue;
                }
                if slot.field == *candidate || slot.pending_field == Some(*candidate) {
                    return false;
                }
            }

            true
        })
        .collect();

    candidates.shuffle(rng);
    candidates.into_iter().next()
}

fn start_info_slot_transition(
    slot_index: usize,
    slots: &mut [InfoSlot; INFO_ROWS * 2],
    elapsed_secs: u64,
    rng: &mut impl Rng,
) {
    let Some(next_field) = pick_unique_next_info_field(slot_index, slots, rng) else {
        slots[slot_index].next_switch_at = schedule_next_info_switch(elapsed_secs, rng);
        return;
    };

    let slot = &mut slots[slot_index];
    slot.pending_field = Some(next_field);
    slot.transition = InfoTransition::FlashOut;
    slot.transition_frames_left = random_flash_out_frames(rng);
    slot.blink_counter = 0;
    slot.flash_color_phase = rng.random_range(0..3);
    slot.is_visible = false;
    slot.next_switch_at = elapsed_secs;
}

fn tick_info_slot_transition(slot: &mut InfoSlot, elapsed_secs: u64, rng: &mut impl Rng) {
    if slot.transition == InfoTransition::Stable {
        return;
    }

    slot.flash_color_phase = (slot.flash_color_phase + 1) % 3;

    slot.blink_counter = slot.blink_counter.saturating_add(1);
    if slot.blink_counter >= INFO_FLASH_TOGGLE_EVERY_FRAMES {
        slot.blink_counter = 0;
        slot.is_visible = !slot.is_visible;
    }

    if slot.transition_frames_left > 0 {
        slot.transition_frames_left -= 1;
    }

    if slot.transition_frames_left > 0 {
        return;
    }

    match slot.transition {
        InfoTransition::FlashOut => {
            if let Some(next_field) = slot.pending_field.take() {
                slot.field = next_field;
            }
            slot.transition = InfoTransition::FlashIn;
            slot.transition_frames_left = random_flash_in_frames(rng);
            slot.blink_counter = 0;
            slot.is_visible = true;
        }
        InfoTransition::FlashIn => {
            slot.transition = InfoTransition::Stable;
            slot.is_visible = true;
            slot.flash_color_phase = 0;
            slot.next_switch_at = schedule_next_info_switch(elapsed_secs, rng);
        }
        InfoTransition::Stable => {}
    }
}

fn info_flash_color(slot: InfoSlot) -> &'static str {
    match slot.flash_color_phase % 3 {
        0 => COLOR_FLASH_ORANGE,
        1 => COLOR_FLASH_YELLOW,
        _ => COLOR_FLASH_RED,
    }
}

fn randomize_info_slots(rng: &mut impl Rng, now_secs: u64) -> [InfoSlot; INFO_ROWS * 2] {
    let mut fields = INFO_FIELDS;
    fields.shuffle(rng);

    let mut slots = [
        InfoSlot {
            field: InfoField::Os,
            next_switch_at: 0,
            pending_field: None,
            transition: InfoTransition::Stable,
            transition_frames_left: 0,
            blink_counter: 0,
            flash_color_phase: 0,
            is_visible: true,
        };
        INFO_ROWS * 2
    ];

    for i in 0..(INFO_ROWS * 2) {
        slots[i] = InfoSlot {
            field: fields[i],
            next_switch_at: schedule_next_info_switch(now_secs, rng),
            pending_field: None,
            transition: InfoTransition::Stable,
            transition_frames_left: 0,
            blink_counter: 0,
            flash_color_phase: 0,
            is_visible: true,
        };
    }

    slots
}

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn push_load_sample(history: &mut VecDeque<f32>, sample: f32) {
    if history.len() >= LOAD_GRAPH_WIDTH {
        history.pop_front();
    }
    history.push_back(sample);
}

fn format_load_graph(history: &VecDeque<f32>, style: CharStyle) -> String {
    if history.is_empty() {
        return "n/a".to_string();
    }

    let mut out = String::with_capacity(LOAD_GRAPH_WIDTH);
    out.extend(std::iter::repeat_n('.', LOAD_GRAPH_WIDTH.saturating_sub(history.len())));

    let chars = load_graph_chars(style);
    for sample in history {
        let normalized = sample.clamp(0.0, 1.0);
        let ch = if normalized < 0.25 {
            chars[0]
        } else if normalized < 0.50 {
            chars[1]
        } else if normalized < 0.75 {
            chars[2]
        } else {
            chars[3]
        };
        out.push(ch);
    }

    out
}

fn render_info_object(slot: InfoSlot, snapshot: &InfoSnapshot<'_>, max_width: usize) -> (String, usize) {
    if max_width == 0 {
        return (String::new(), 0);
    }

    let label_text = format!("{}: ", slot.field.label());
    let label_width = label_text.chars().count();
    let value = slot.field.value(snapshot);
    let value_width = value.chars().count();

    let used = (label_width + value_width).min(max_width);
    if !slot.is_visible {
        return (" ".repeat(used), used);
    }

    let label_color = if slot.transition == InfoTransition::Stable {
        COLOR_INFO_BAR
    } else {
        info_flash_color(slot)
    };
    let value_color = if slot.transition == InfoTransition::Stable {
        COLOR_GRAY
    } else {
        info_flash_color(slot)
    };

    let mut out = String::new();
    out.push_str(label_color);

    if label_width >= max_width {
        let clipped = truncate_to_width(&label_text, max_width);
        let used = clipped.chars().count();
        out.push_str(&clipped);
        out.push_str(COLOR_RESET);
        return (out, used);
    }

    out.push_str(&label_text);
    out.push_str(value_color);
    let clipped_value = truncate_to_width(&value, max_width - label_width);
    let clipped_value_width = clipped_value.chars().count();
    out.push_str(&clipped_value);
    out.push_str(COLOR_RESET);
    (out, label_width + clipped_value_width)
}

fn render_info_group(slots: &[InfoSlot], snapshot: &InfoSnapshot<'_>, max_width: usize) -> (String, usize) {
    if max_width == 0 {
        return (String::new(), 0);
    }

    let mut out = String::new();
    let mut used = 0usize;
    for slot in slots {
        if used >= max_width {
            break;
        }

        if used > 0 {
            if used + 2 > max_width {
                break;
            }
            out.push_str("  ");
            used += 2;
        }

        let remaining = max_width - used;
        let (obj, obj_used) = render_info_object(*slot, snapshot, remaining);
        if obj_used == 0 {
            break;
        }
        out.push_str(&obj);
        used += obj_used;
    }

    (out, used)
}

fn apply_command(
    command: &str,
    paused: &mut bool,
    style: &mut CharStyle,
    color: &mut ColorTheme,
    program: &mut ProgramMode,
    preferences_dirty: &mut bool,
) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }

    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or_default();

    match cmd {
        "help" => Some(
            "Commands: help, style, color, program, p, pause, resume, clear, quit"
                .to_string(),
        ),
        "clear" => Some(String::new()),
        "quit" | "exit" | "q" => None,
        "p" | "pause" => {
            *paused = true;
            Some("Program paused".to_string())
        }
        "resume" => {
            *paused = false;
            Some("Program resumed".to_string())
        }
        "style" => {
            let Some(val) = parts.next() else {
                return Some(format!(
                    "Current style: {} (available: braille, block, binary, hex)",
                    style.as_str()
                ));
            };

            match CharStyle::parse(val) {
                Some(next_style) => {
                    *style = next_style;
                    *preferences_dirty = true;
                    Some(format!("Style set to {}", style.as_str()))
                }
                None => Some("Unknown style. Use: braille, block, binary, hex".to_string()),
            }
        }
        "color" => {
            let Some(val) = parts.next() else {
                return Some(format!(
                    "Current color: {} (available: green, blue, cyan, yellow, red, magenta, orange, white, gray)",
                    color.as_str()
                ));
            };

            match ColorTheme::parse(val) {
                Some(next_color) => {
                    *color = next_color;
                    *preferences_dirty = true;
                    Some(format!("Color set to {}", color.as_str()))
                }
                None => Some("Unknown color. Use: green, blue, cyan, yellow, red, magenta, orange, white, gray".to_string()),
            }
        }
        "program" => {
            let Some(val) = parts.next() else {
                return Some(format!("Current program: {} (available: rain, vortex, circuit, usage)", program.as_str()));
            };

            match ProgramMode::parse(val) {
                Some(next_program) => {
                    *program = next_program;
                    *preferences_dirty = true;
                    Some(format!("Program set to {}", program.as_str()))
                }
                None => Some("Unknown program. Use: rain, vortex, circuit, usage".to_string()),
            }
        }
        _ => Some(format!("Unknown command: {}", trimmed)),
    }
}

fn ensure_buffers(buffers: &mut RenderBuffers, width: u16, height: u16) {
    let w = width as usize;
    let h = height as usize;
    let screen_len = w * h;
    if buffers.front_screen.len() != screen_len {
        buffers.front_screen = vec![' '; screen_len];
    }
    if buffers.back_screen.len() != screen_len {
        buffers.back_screen = vec![' '; screen_len];
    }

    let output_len =
        SYNC_UPDATE_BEGIN.len() + CURSOR_HOME.len() + (w * h) + h.saturating_sub(1) + (w * 2) + 64 + SYNC_UPDATE_END.len();
    if buffers.output.capacity() < output_len {
        buffers.output.reserve(output_len - buffers.output.capacity());
    }
}

fn draw_frame(
    stdout: &mut Stdout,
    streams: &[Stream],
    usage_samples: &[f32],
    width: u16,
    height: u16,
    rng: &mut impl Rng,
    buffers: &mut RenderBuffers,
    command_input: &str,
    status_line: &str,
    animate_scene: bool,
    style: CharStyle,
    color: ColorTheme,
    program: ProgramMode,
    pattern_phase: f32,
    info_slots: &[InfoSlot; INFO_ROWS * 2],
    info_snapshot: &InfoSnapshot<'_>,
) -> io::Result<()> {
    let w = width as usize;
    let rain_h = height.saturating_sub(1) as usize;
    ensure_buffers(buffers, width, height.saturating_sub(1));
    if animate_scene {
        render_scene(
            &mut buffers.back_screen,
            w,
            rain_h,
            streams,
            usage_samples,
            style,
            program,
            pattern_phase,
            rng,
        );

        std::mem::swap(&mut buffers.front_screen, &mut buffers.back_screen);
    }

    buffers.output.clear();
    buffers.output.push_str(SYNC_UPDATE_BEGIN);
    buffers.output.push_str(CURSOR_HOME);

    for y in 0..rain_h {
        let row = &buffers.front_screen[y * w..(y + 1) * w];
        let mut current_band: Option<usize> = None;
        for ch in row {
            if *ch != ' ' {
                let band = color_band_for_char(style, *ch);
                if current_band != Some(band) {
                    buffers.output.push_str(color.shade_color_code(band));
                    current_band = Some(band);
                }
            }
            buffers.output.push(*ch);
        }
        if y < rain_h - 1 {
            buffers.output.push('\n');
        }
    }
    buffers.output.push_str(COLOR_RESET);

    if height > 0 {
        let gap = 2usize;
        let left_w = if w > gap { (w - gap) / 2 } else { w };
        let right_w = w.saturating_sub(left_w + gap);

        buffers.output.push_str("\x1b[1;1H");
        buffers.output.extend(std::iter::repeat_n(' ', w));

        let (left_group, _) = render_info_group(&info_slots[..INFO_ROWS], info_snapshot, left_w);
        buffers.output.push_str("\x1b[1;1H");
        buffers.output.push_str(&left_group);

        if right_w > 0 {
            let (right_group, right_used) = render_info_group(&info_slots[INFO_ROWS..], info_snapshot, right_w);
            let right_col = (left_w + gap + 1) + right_w.saturating_sub(right_used);
            buffers.output.push_str("\x1b[1;");
            buffers.output.push_str(&right_col.to_string());
            buffers.output.push_str("H");
            buffers.output.push_str(&right_group);
        }
    }

    if height > 0 {
        buffers.output.push_str("\x1b[");
        buffers.output.push_str(&(height as usize).to_string());
        buffers.output.push_str(";1H");
    }

    let prompt_line = build_prompt_line(
        w,
        command_input,
        status_line,
        COLOR_GRAY,
        COLOR_CYAN,
        COLOR_RESET,
    );
    buffers.output.push_str(&prompt_line);
    buffers.output.push_str(SYNC_UPDATE_END);

    stdout.write_all(buffers.output.as_bytes())?;
    stdout.flush()
}

fn run(stdout: &mut Stdout) -> io::Result<()> {
    let mut rng = rand::rng();
    let start_time = Instant::now();
    let distro_name = read_distro_name();
    let hostname = read_hostname();
    let kernel_release = read_kernel_release();
    let username = read_username();
    let shell_name = read_shell_name();
    let process_id = std::process::id();
    let host_uptime_at_start = read_host_uptime_secs();
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut cpu_sampler = CpuUsageSampler::default();
    let mut load_history = VecDeque::with_capacity(LOAD_GRAPH_WIDTH);
    let mut size = terminal::size()?;
    let mut streams: Vec<Stream> = (0..size.0)
        .map(|_| random_stream(size.1.saturating_sub(1), DEFAULT_FPS, &mut rng))
        .collect();
    let mut buffers = RenderBuffers {
        front_screen: Vec::new(),
        back_screen: Vec::new(),
        output: String::new(),
    };
    let mut command_input = String::new();
    let mut status_line = "Type 'help' for more options.".to_string();
    let mut display_refresh_hz = detect_display_refresh_hz().unwrap_or(DEFAULT_FPS).clamp(30, 240);
    let mut fps = display_refresh_hz;
    let mut frame_time = Duration::from_millis(1000 / fps.max(1));
    let mut next_refresh_probe = Instant::now() + Duration::from_secs(5);
    let mut should_exit = false;
    let mut paused = false;
    let prefs = load_preferences();
    let mut style = prefs.style;
    let mut color = prefs.color;
    let mut program = prefs.program;
    let mut pattern_phase = 0.0f32;
    let mut info_slots = randomize_info_slots(&mut rng, 0);

    loop {
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        should_exit = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        should_exit = true;
                    }
                    KeyCode::Backspace => {
                        command_input.pop();
                    }
                    KeyCode::Enter => {
                        let mut preferences_dirty = false;
                        match apply_command(
                            &command_input,
                            &mut paused,
                            &mut style,
                            &mut color,
                            &mut program,
                            &mut preferences_dirty,
                        ) {
                            Some(next_status) => status_line = next_status,
                            None => should_exit = true,
                        }
                        if preferences_dirty {
                            save_preferences(Preferences {
                                style,
                                color,
                                program,
                            });
                        }
                        command_input.clear();
                    }
                    KeyCode::Tab => {
                        if let Some(completed) = complete_command_input(&command_input) {
                            command_input = completed;
                        }
                    }
                    KeyCode::Char(ch)
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT)
                            && !ch.is_control() =>
                    {
                        command_input.push(ch);
                    }
                    _ => {}
                }

                if should_exit {
                    break;
                }
            }
        }

        if should_exit {
            break;
        }

        let current = terminal::size()?;
        if current != size {
            size = current;
            display_refresh_hz = detect_display_refresh_hz().unwrap_or(display_refresh_hz).clamp(30, 240);
            fps = display_refresh_hz;
            frame_time = Duration::from_millis(1000 / fps.max(1));
            streams = (0..size.0)
                .map(|_| random_stream(size.1.saturating_sub(1), fps, &mut rng))
                .collect();
        }

        if Instant::now() >= next_refresh_probe {
            if let Some(hz) = detect_display_refresh_hz() {
                let hz = hz.clamp(30, 240);
                if hz != display_refresh_hz {
                    display_refresh_hz = hz;
                    fps = display_refresh_hz;
                    frame_time = Duration::from_millis(1000 / fps.max(1));
                }
            }
            next_refresh_probe = Instant::now() + Duration::from_secs(5);
        }

        if !paused {
            pattern_phase += 0.18 * (BASE_ANIM_FPS / fps.max(1) as f32);

            if program == ProgramMode::Rain {
                update_rain_streams(&mut streams, size.1.saturating_sub(1), fps, &mut rng);
            }
        }

        let elapsed_secs = start_time.elapsed().as_secs();
        for idx in 0..info_slots.len() {
            if info_slots[idx].transition == InfoTransition::Stable && elapsed_secs >= info_slots[idx].next_switch_at {
                start_info_slot_transition(idx, &mut info_slots, elapsed_secs, &mut rng);
            }

            let slot = &mut info_slots[idx];
            tick_info_slot_transition(slot, elapsed_secs, &mut rng);
        }

        let host_uptime_text = host_uptime_at_start
            .map(|start| format_uptime(start + elapsed_secs))
            .unwrap_or_else(|| "--d --h --m --s".to_string());
        let app_uptime_text = format_uptime(elapsed_secs);
        if let Some(load_sample) = read_cpu_usage_sample(&mut cpu_sampler) {
            push_load_sample(&mut load_history, load_sample);
        }
        let load_graph_text = format_load_graph(&load_history, style);
        let usage_samples: Vec<f32> = load_history.iter().copied().collect();
        let memory_usage_text = read_memory_usage().unwrap_or_else(|| "n/a".to_string());
        let network_text = read_network_summary();
        let process_memory_text = read_process_rss().unwrap_or_else(|| "n/a".to_string());
        let info_snapshot = InfoSnapshot {
            distro: &distro_name,
            hostname: &hostname,
            kernel_release: &kernel_release,
            username: &username,
            shell: &shell_name,
            network: &network_text,
            host_uptime: &host_uptime_text,
            app_uptime: &app_uptime_text,
            load_graph: &load_graph_text,
            memory_usage: &memory_usage_text,
            pid: process_id,
            process_memory: &process_memory_text,
            style,
            fps,
            width: size.0,
            height: size.1,
            columns: streams.len(),
            cpu_cores,
        };

        draw_frame(
            stdout,
            &streams,
            &usage_samples,
            size.0,
            size.1,
            &mut rng,
            &mut buffers,
            &command_input,
            &status_line,
            !paused,
            style,
            color,
            program,
            pattern_phase,
            &info_slots,
            &info_snapshot,
        )?;
        thread::sleep(frame_time);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let result = run(&mut stdout);

    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}
