use std::fs;
use std::path::PathBuf;

use crate::patterns::{CharStyle, ColorTheme, ProgramMode};

#[derive(Clone, Copy)]
pub struct Preferences {
    pub style: CharStyle,
    pub color: ColorTheme,
    pub program: ProgramMode,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            style: CharStyle::Braille,
            color: ColorTheme::Green,
            program: ProgramMode::Rain,
        }
    }
}

fn preferences_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let mut p = PathBuf::from(home);
                p.push(".config");
                p
            })
        })?;

    let mut path = base;
    path.push("fabric");
    path.push("preferences.conf");
    Some(path)
}

pub fn load_preferences() -> Preferences {
    let Some(path) = preferences_path() else {
        return Preferences::default();
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Preferences::default(),
    };

    let mut prefs = Preferences::default();
    for line in content.lines() {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap_or_default().trim();
        let val = parts.next().unwrap_or_default().trim();

        match key {
            "style" => {
                if let Some(style) = CharStyle::parse(val) {
                    prefs.style = style;
                }
            }
            "color" => {
                if let Some(color) = ColorTheme::parse(val) {
                    prefs.color = color;
                }
            }
            "program" => {
                if let Some(program) = ProgramMode::parse(val) {
                    prefs.program = program;
                }
            }
            _ => {}
        }
    }

    prefs
}

pub fn save_preferences(prefs: Preferences) {
    let Some(path) = preferences_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let body = format!(
        "style={}\ncolor={}\nprogram={}\n",
        prefs.style.as_str(),
        prefs.color.as_str(),
        prefs.program.as_str()
    );

    let _ = fs::write(path, body);
}
