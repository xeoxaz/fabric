const COMMAND_PREFIX: &str = "Command: ";
const COMMANDS: [&str; 10] = [
    "help", "style", "color", "program", "p", "pause", "resume", "clear", "quit", "exit",
];

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn push_visible_segment(line: &mut String, text: &str, remaining: &mut usize) {
    if *remaining == 0 {
        return;
    }

    let segment = truncate_to_width(text, *remaining);
    let used = segment.chars().count();
    line.push_str(&segment);
    *remaining = remaining.saturating_sub(used);
}

fn unique_prefix_match<'a>(prefix: &str, options: &'a [&'a str]) -> Option<&'a str> {
    let mut matched: Option<&str> = None;
    for option in options {
        if option.starts_with(prefix) {
            if matched.is_some() {
                return None;
            }
            matched = Some(*option);
        }
    }
    matched
}

fn command_suggestion(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains(' ') {
        let cmd = trimmed.split_whitespace().next().unwrap_or_default();
        let rest = trimmed.strip_prefix(cmd).unwrap_or_default();
        if cmd == "style" && rest.trim().is_empty() {
            return Some("<braille|block|binary|hex>".to_string());
        }
        if cmd == "color" && rest.trim().is_empty() {
            return Some("<green|blue|cyan|yellow|red|magenta|orange|white|gray>".to_string());
        }
        if cmd == "program" && rest.trim().is_empty() {
            return Some("<rain|vortex|circuit|usage>".to_string());
        }

        let args: Vec<&str> = rest.split_whitespace().collect();
        if cmd == "style" && args.len() == 1 {
            let options = ["braille", "block", "binary", "hex"];
            if let Some(matched) = unique_prefix_match(args[0], &options) {
                if matched != args[0] {
                    return Some(matched[args[0].len()..].to_string());
                }
            }
        }
        if cmd == "color" && args.len() == 1 {
            let options = ["green", "blue", "cyan", "yellow", "red", "magenta", "orange", "white", "gray"];
            if let Some(matched) = unique_prefix_match(args[0], &options) {
                if matched != args[0] {
                    return Some(matched[args[0].len()..].to_string());
                }
            }
        }
        if cmd == "program" && args.len() == 1 {
            let options = ["rain", "vortex", "circuit", "usage"];
            if let Some(matched) = unique_prefix_match(args[0], &options) {
                if matched != args[0] {
                    return Some(matched[args[0].len()..].to_string());
                }
            }
        }

        return None;
    }

    for cmd in COMMANDS {
        if cmd.starts_with(trimmed) && cmd != trimmed {
            return Some(cmd[trimmed.len()..].to_string());
        }
    }

    if trimmed == "style" {
        return Some(" <braille|block|binary|hex>".to_string());
    }
    if trimmed == "color" {
        return Some(" <green|blue|cyan|yellow|red|magenta|orange|white|gray>".to_string());
    }
    if trimmed == "program" {
        return Some(" <rain|vortex|circuit|usage>".to_string());
    }

    None
}

pub fn complete_command_input(input: &str) -> Option<String> {
    let trimmed_start = input.trim_start();
    if trimmed_start.is_empty() {
        return None;
    }

    if !trimmed_start.contains(' ') {
        let matched = unique_prefix_match(trimmed_start, &COMMANDS)?;
        if matched == trimmed_start {
            return None;
        }
        return Some(matched.to_string());
    }

    let mut parts = trimmed_start.split_whitespace();
    let cmd = parts.next().unwrap_or_default();
    let args: Vec<&str> = parts.collect();

    match cmd {
        "style" => {
            let options = ["braille", "block", "binary", "hex"];
            let current = args.first().copied().unwrap_or_default();
            let matched = unique_prefix_match(current, &options)?;
            if matched == current {
                return None;
            }
            Some(format!("style {}", matched))
        }
        "color" => {
            let options = ["green", "blue", "cyan", "yellow", "red", "magenta", "orange", "white", "gray"];
            let current = args.first().copied().unwrap_or_default();
            let matched = unique_prefix_match(current, &options)?;
            if matched == current {
                return None;
            }
            Some(format!("color {}", matched))
        }
        "program" => {
            let options = ["rain", "vortex", "circuit", "usage"];
            let current = args.first().copied().unwrap_or_default();
            let matched = unique_prefix_match(current, &options)?;
            if matched == current {
                return None;
            }
            Some(format!("program {}", matched))
        }
        _ => None,
    }
}

pub fn build_prompt_line(
    width: usize,
    input: &str,
    placeholder: &str,
    color_gray: &str,
    color_cyan: &str,
    color_reset: &str,
) -> String {
    if width == 0 {
        return String::new();
    }

    let prefix_width = COMMAND_PREFIX.chars().count();
    if width <= prefix_width {
        return truncate_to_width(COMMAND_PREFIX, width);
    }

    let mut line = String::new();
    line.push_str(COMMAND_PREFIX);
    let mut remaining = width - prefix_width;

    if input.is_empty() {
        line.push_str(color_gray);
        push_visible_segment(&mut line, placeholder, &mut remaining);
        line.push_str(color_reset);
        line.extend(std::iter::repeat_n(' ', remaining));
        return line;
    }

    push_visible_segment(&mut line, input, &mut remaining);

    if let Some(suggestion) = command_suggestion(input) {
        if !suggestion.is_empty() && remaining > 0 {
            let is_subarg_context = input.trim_start().contains(' ');
            if is_subarg_context {
                line.push_str(color_cyan);
            } else {
                line.push_str(color_gray);
            }
            push_visible_segment(&mut line, &suggestion, &mut remaining);
            line.push_str(color_reset);
        }
    }

    line.extend(std::iter::repeat_n(' ', remaining));
    line
}
