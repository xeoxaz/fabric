use rand::Rng;

const BLOCK_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
const BINARY_CHARS: &[char] = &['0', '1'];
const HEX_CHARS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
const BRAILLE_START: u32 = 0x2801;
const BRAILLE_END: u32 = 0x28FF;

#[derive(Clone, Copy)]
pub enum CharStyle {
    Braille,
    Block,
    Binary,
    Hex,
}

impl CharStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            CharStyle::Braille => "braille",
            CharStyle::Block => "block",
            CharStyle::Binary => "binary",
            CharStyle::Hex => "hex",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "braille" => Some(CharStyle::Braille),
            "block" => Some(CharStyle::Block),
            "binary" => Some(CharStyle::Binary),
            "hex" => Some(CharStyle::Hex),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorTheme {
    Green,
    Blue,
    Cyan,
    Yellow,
    Red,
    Magenta,
    Orange,
    White,
    Gray,
}

impl ColorTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ColorTheme::Green => "green",
            ColorTheme::Blue => "blue",
            ColorTheme::Cyan => "cyan",
            ColorTheme::Yellow => "yellow",
            ColorTheme::Red => "red",
            ColorTheme::Magenta => "magenta",
            ColorTheme::Orange => "orange",
            ColorTheme::White => "white",
            ColorTheme::Gray => "gray",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "green" => Some(ColorTheme::Green),
            "blue" => Some(ColorTheme::Blue),
            "cyan" => Some(ColorTheme::Cyan),
            "yellow" => Some(ColorTheme::Yellow),
            "red" => Some(ColorTheme::Red),
            "magenta" => Some(ColorTheme::Magenta),
            "orange" => Some(ColorTheme::Orange),
            "white" => Some(ColorTheme::White),
            "gray" => Some(ColorTheme::Gray),
            _ => None,
        }
    }

    pub fn color_code(self) -> &'static str {
        match self {
            ColorTheme::Green => "\x1b[38;5;46m",
            ColorTheme::Blue => "\x1b[38;5;39m",
            ColorTheme::Cyan => "\x1b[38;5;51m",
            ColorTheme::Yellow => "\x1b[38;5;220m",
            ColorTheme::Red => "\x1b[38;5;203m",
            ColorTheme::Magenta => "\x1b[38;5;201m",
            ColorTheme::Orange => "\x1b[38;5;208m",
            ColorTheme::White => "\x1b[38;5;250m",
            ColorTheme::Gray => "\x1b[38;5;244m",
        }
    }

    pub fn shade_color_code(self, band: usize) -> &'static str {
        match self {
            ColorTheme::Green => match band.min(5) {
                0 => "\x1b[38;5;22m",
                1 => "\x1b[38;5;28m",
                2 => self.color_code(),
                3 => "\x1b[38;5;82m",
                4 => "\x1b[38;5;120m",
                _ => "\x1b[38;5;157m",
            },
            ColorTheme::Blue => match band.min(5) {
                0 => "\x1b[38;5;17m",
                1 => "\x1b[38;5;18m",
                2 => self.color_code(),
                3 => "\x1b[38;5;45m",
                4 => "\x1b[38;5;81m",
                _ => "\x1b[38;5;117m",
            },
            ColorTheme::Cyan => match band.min(5) {
                0 => "\x1b[38;5;23m",
                1 => "\x1b[38;5;30m",
                2 => self.color_code(),
                3 => "\x1b[38;5;87m",
                4 => "\x1b[38;5;123m",
                _ => "\x1b[38;5;159m",
            },
            ColorTheme::Yellow => match band.min(5) {
                0 => "\x1b[38;5;94m",
                1 => "\x1b[38;5;136m",
                2 => self.color_code(),
                3 => "\x1b[38;5;228m",
                4 => "\x1b[38;5;229m",
                _ => "\x1b[38;5;230m",
            },
            ColorTheme::Red => match band.min(5) {
                0 => "\x1b[38;5;52m",
                1 => "\x1b[38;5;88m",
                2 => self.color_code(),
                3 => "\x1b[38;5;210m",
                4 => "\x1b[38;5;217m",
                _ => "\x1b[38;5;224m",
            },
            ColorTheme::Magenta => match band.min(5) {
                0 => "\x1b[38;5;54m",
                1 => "\x1b[38;5;91m",
                2 => self.color_code(),
                3 => "\x1b[38;5;207m",
                4 => "\x1b[38;5;213m",
                _ => "\x1b[38;5;219m",
            },
            ColorTheme::Orange => match band.min(5) {
                0 => "\x1b[38;5;130m",
                1 => "\x1b[38;5;166m",
                2 => self.color_code(),
                3 => "\x1b[38;5;214m",
                4 => "\x1b[38;5;220m",
                _ => "\x1b[38;5;227m",
            },
            ColorTheme::White => match band.min(5) {
                0 => "\x1b[38;5;238m",
                1 => "\x1b[38;5;245m",
                2 => self.color_code(),
                3 => "\x1b[38;5;252m",
                4 => "\x1b[38;5;255m",
                _ => "\x1b[38;5;231m",
            },
            ColorTheme::Gray => match band.min(5) {
                0 => "\x1b[38;5;236m",
                1 => "\x1b[38;5;240m",
                2 => self.color_code(),
                3 => "\x1b[38;5;247m",
                4 => "\x1b[38;5;250m",
                _ => "\x1b[38;5;254m",
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgramMode {
    Rain,
    Vortex,
    Circuit,
    Usage,
}

impl ProgramMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ProgramMode::Rain => "rain",
            ProgramMode::Vortex => "vortex",
            ProgramMode::Circuit => "circuit",
            ProgramMode::Usage => "usage",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "rain" => Some(ProgramMode::Rain),
            "vortex" => Some(ProgramMode::Vortex),
            "circuit" => Some(ProgramMode::Circuit),
            "usage" => Some(ProgramMode::Usage),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Stream {
    pub head: f32,
    pub speed: f32,
    pub length: usize,
    pub spawn_delay_frames: u16,
    pub fall_tick_frames: u8,
    pub fall_tick_counter: u8,
}

fn random_braille(rng: &mut impl Rng) -> char {
    let code = rng.random_range(BRAILLE_START..=BRAILLE_END);
    char::from_u32(code).unwrap_or('⣿')
}

fn random_styled_char(style: CharStyle, rng: &mut impl Rng) -> char {
    match style {
        CharStyle::Braille => random_braille(rng),
        CharStyle::Block => BLOCK_CHARS[rng.random_range(0..BLOCK_CHARS.len())],
        CharStyle::Binary => BINARY_CHARS[rng.random_range(0..BINARY_CHARS.len())],
        CharStyle::Hex => HEX_CHARS[rng.random_range(0..HEX_CHARS.len())],
    }
}

pub fn random_stream(height: u16, fps: u64, rng: &mut impl Rng) -> Stream {
    let max_delay = ((fps as u16).saturating_mul(2)).max(1);
    let fall_tick_frames = rng.random_range(1..=3);
    Stream {
        head: -(height as f32),
        speed: rng.random_range(0.25..1.1),
        length: rng.random_range(6..=(height.max(8) as usize / 2)),
        spawn_delay_frames: rng.random_range(0..=max_delay),
        fall_tick_frames,
        fall_tick_counter: rng.random_range(0..fall_tick_frames),
    }
}

pub fn update_rain_streams(streams: &mut [Stream], height: u16, fps: u64, rng: &mut impl Rng) {
    let frame_scale = (30.0f32 / fps.max(1) as f32).clamp(0.25, 4.0);

    for stream in streams {
        if stream.spawn_delay_frames > 0 {
            stream.spawn_delay_frames -= 1;
            continue;
        }

        if stream.fall_tick_counter > 0 {
            stream.fall_tick_counter -= 1;
            continue;
        }
        stream.fall_tick_counter = stream.fall_tick_frames.saturating_sub(1);

        stream.head += stream.speed * frame_scale;
        if stream.head - stream.length as f32 > height.saturating_sub(1) as f32 {
            *stream = random_stream(height, fps, rng);
        }
    }
}

fn render_vortex_pattern(
    screen: &mut [char],
    width: usize,
    height: usize,
    phase: f32,
    style: CharStyle,
    rng: &mut impl Rng,
) {
    if width == 0 || height == 0 {
        return;
    }

    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let radius = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let arm = (angle * 4.0 + radius * 0.24 - phase * 1.7).sin();
            let ring = (radius * 0.35 - phase * 1.2).cos();
            let intensity = arm * 0.75 + ring * 0.25;

            if intensity > 0.72 || (radius < 3.0 && rng.random_bool(0.55)) {
                let idx = y * width + x;
                screen[idx] = random_styled_char(style, rng);
            }
        }
    }
}

fn render_circuit_pattern(
    screen: &mut [char],
    width: usize,
    height: usize,
    phase: f32,
    style: CharStyle,
    rng: &mut impl Rng,
) {
    if width == 0 || height == 0 {
        return;
    }

    let phase_i = (phase * 5.0) as i32;

    let lane_h = 6i32;
    let lane_v = 14i32;

    for y in 0..height {
        for x in 0..width {
            let xi = x as i32;
            let yi = y as i32;

            let h_line = yi.rem_euclid(lane_h) == 0;
            let v_line = xi.rem_euclid(lane_v) == 0;
            let node = h_line && v_line;

            let h_pulse = h_line && ((xi + phase_i * 2).rem_euclid(18) == 0);
            let v_pulse = v_line && ((yi - phase_i * 3).rem_euclid(16) == 0);

            let bridge = (yi.rem_euclid(lane_h) == 3)
                && ((xi + yi + phase_i).rem_euclid(lane_v * 2) == 0);
            let sparkle = node && rng.random_bool(0.08);

            let bright = node || h_pulse || v_pulse || bridge || sparkle;
            if bright {
                let idx = y * width + x;
                screen[idx] = random_styled_char(style, rng);
            }
        }
    }
}

fn render_usage_pattern(
    screen: &mut [char],
    width: usize,
    height: usize,
    _phase: f32,
    style: CharStyle,
    usage_samples: &[f32],
    rng: &mut impl Rng,
) {
    if width == 0 || height == 0 {
        return;
    }

    let graph_chars = load_graph_chars(style);
    let h = height as f32;
    let sample_count = usage_samples.len();

    for y in 0..height {
        if y % 4 == 0 {
            for x in 0..width {
                let idx = y * width + x;
                if rng.random_bool(0.035) {
                    screen[idx] = graph_chars[0];
                }
            }
        }
    }

    for x in 0..width {
        let usage = if sample_count == 0 {
            0.5
        } else {
            let pos = if width <= 1 {
                0.0
            } else {
                (x as f32 / (width - 1) as f32) * (sample_count.saturating_sub(1) as f32)
            };
            let i0 = pos.floor() as usize;
            let i1 = (i0 + 1).min(sample_count - 1);
            let t = pos - i0 as f32;
            let s0 = usage_samples[i0];
            let s1 = usage_samples[i1];
            (s0 + (s1 - s0) * t).clamp(0.04, 0.98)
        };

        let line_y = ((1.0 - usage) * (h - 1.0)).round() as usize;
        for y in line_y..height {
            let idx = y * width + x;
            let level = (y - line_y) as f32 / (height.max(1) as f32);
            screen[idx] = if level < 0.12 {
                graph_chars[3]
            } else if level < 0.28 {
                graph_chars[2]
            } else if level < 0.46 {
                graph_chars[1]
            } else {
                graph_chars[0]
            };
        }

        if line_y > 0 {
            let idx = (line_y - 1) * width + x;
            if rng.random_bool(0.18) {
                screen[idx] = random_styled_char(style, rng);
            }
        }
    }
}

fn apply_global_shading(screen: &mut [char], width: usize, height: usize, style: CharStyle) {
    if width == 0 || height == 0 {
        return;
    }

    let src = screen.to_vec();
    let shades = load_graph_chars(style);

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if src[idx] != ' ' {
                continue;
            }

            let mut neighbors = 0u8;
            let y0 = y.saturating_sub(1);
            let y1 = (y + 1).min(height - 1);
            let x0 = x.saturating_sub(1);
            let x1 = (x + 1).min(width - 1);

            for ny in y0..=y1 {
                for nx in x0..=x1 {
                    if nx == x && ny == y {
                        continue;
                    }
                    if src[ny * width + nx] != ' ' {
                        neighbors = neighbors.saturating_add(1);
                    }
                }
            }

            if neighbors >= 5 {
                screen[idx] = shades[2];
            } else if neighbors >= 3 {
                screen[idx] = shades[1];
            } else if neighbors >= 2 {
                screen[idx] = shades[0];
            }
        }
    }
}

pub fn render_scene(
    screen: &mut [char],
    width: usize,
    height: usize,
    streams: &[Stream],
    usage_samples: &[f32],
    style: CharStyle,
    program: ProgramMode,
    pattern_phase: f32,
    rng: &mut impl Rng,
) {
    screen.fill(' ');

    match program {
        ProgramMode::Rain => {
            for (x, stream) in streams.iter().enumerate() {
                if stream.spawn_delay_frames > 0 {
                    continue;
                }

                let head = stream.head.floor() as i32;
                for i in 0..stream.length {
                    let y = head - i as i32;
                    if y >= 0 && (y as usize) < height {
                        let idx = y as usize * width + x;
                        screen[idx] = random_styled_char(style, rng);
                    }
                }
            }
        }
        ProgramMode::Vortex => {
            render_vortex_pattern(screen, width, height, pattern_phase, style, rng);
        }
        ProgramMode::Circuit => {
            render_circuit_pattern(screen, width, height, pattern_phase, style, rng);
        }
        ProgramMode::Usage => {
            render_usage_pattern(screen, width, height, pattern_phase, style, usage_samples, rng);
        }
    }

    if program != ProgramMode::Rain {
        apply_global_shading(screen, width, height, style);
    }
}

pub fn load_graph_chars(style: CharStyle) -> [char; 4] {
    match style {
        CharStyle::Braille => ['.', '⠂', '⠒', '⣿'],
        CharStyle::Block => ['.', '-', '*', '#'],
        CharStyle::Binary => ['0', '0', '1', '1'],
        CharStyle::Hex => ['1', '3', '7', 'F'],
    }
}

pub fn color_band_for_char(style: CharStyle, ch: char) -> usize {
    if ch == ' ' {
        return 0;
    }

    match style {
        CharStyle::Braille => {
            if ch == '.' {
                return 0;
            }

            let code = ch as u32;
            if (0x2800..=0x28FF).contains(&code) {
                let dots = (code - 0x2800).count_ones() as usize;
                return (dots * 5) / 8;
            }

            3
        }
        CharStyle::Block => {
            let idx = BLOCK_CHARS.iter().position(|c| *c == ch).unwrap_or(BLOCK_CHARS.len() - 1);
            (idx * 5) / (BLOCK_CHARS.len() - 1)
        }
        CharStyle::Binary => {
            if ch == '0' {
                1
            } else if ch == '1' {
                5
            } else {
                3
            }
        }
        CharStyle::Hex => {
            if let Some(val) = ch.to_digit(16) {
                (val as usize * 5) / 15
            } else {
                3
            }
        }
    }
}
