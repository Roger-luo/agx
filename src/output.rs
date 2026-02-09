use std::{
    fmt::Display,
    io::{self, IsTerminal, Write},
};

use ratatui::{
    crossterm::{
        execute,
        style::{
            Attribute, Color as CrosstermColor, Print, ResetColor, SetAttribute,
            SetBackgroundColor, SetForegroundColor, force_color_output,
        },
    },
    style::{Color, Modifier, Style},
};

#[derive(Clone, Copy)]
enum MessageKind {
    Path,
    Log,
    Hint,
    Quote,
    Warning,
    Error,
}

pub(crate) fn print_path(path: impl Display) {
    write_stdout(path.to_string(), MessageKind::Path);
}

pub(crate) fn print_log(message: impl AsRef<str>) {
    let text = format!("log: {}", message.as_ref());
    write_stdout(text, MessageKind::Log);
}

pub(crate) fn print_hint(message: impl AsRef<str>) {
    let text = format!("hint: {}", message.as_ref());
    write_stdout(text, MessageKind::Hint);
}

pub(crate) fn print_quote(message: impl AsRef<str>) {
    let text = format!("> {}", message.as_ref());
    write_stdout(text, MessageKind::Quote);
}

pub(crate) fn print_warning(message: impl AsRef<str>) {
    let text = format!("warning: {}", message.as_ref());
    write_stderr(text, MessageKind::Warning);
}

pub(crate) fn print_error(message: impl AsRef<str>) {
    let text = format!("error: {}", message.as_ref());
    write_stderr(text, MessageKind::Error);
}

fn write_stdout(text: String, kind: MessageKind) {
    let mut stdout = io::stdout();
    write_line(&mut stdout, &text, kind, stdout_supports_color());
}

fn write_stderr(text: String, kind: MessageKind) {
    let mut stderr = io::stderr();
    write_line(&mut stderr, &text, kind, stderr_supports_color());
}

fn write_line(writer: &mut impl Write, text: &str, kind: MessageKind, use_color: bool) {
    if use_color && write_colored_line(writer, text, style_for(kind)).is_ok() {
        return;
    }
    let _ = writeln!(writer, "{text}");
}

fn write_colored_line(writer: &mut impl Write, text: &str, style: Style) -> io::Result<()> {
    if let Some(color) = style.fg {
        execute!(writer, SetForegroundColor(CrosstermColor::from(color)))?;
    }
    if let Some(color) = style.bg {
        execute!(writer, SetBackgroundColor(CrosstermColor::from(color)))?;
    }
    apply_modifiers(writer, style.add_modifier)?;

    execute!(
        writer,
        Print(text),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n")
    )?;
    Ok(())
}

fn apply_modifiers(writer: &mut impl Write, modifiers: Modifier) -> io::Result<()> {
    if modifiers.contains(Modifier::BOLD) {
        execute!(writer, SetAttribute(Attribute::Bold))?;
    }
    if modifiers.contains(Modifier::DIM) {
        execute!(writer, SetAttribute(Attribute::Dim))?;
    }
    if modifiers.contains(Modifier::ITALIC) {
        execute!(writer, SetAttribute(Attribute::Italic))?;
    }
    if modifiers.contains(Modifier::UNDERLINED) {
        execute!(writer, SetAttribute(Attribute::Underlined))?;
    }
    if modifiers.contains(Modifier::SLOW_BLINK) {
        execute!(writer, SetAttribute(Attribute::SlowBlink))?;
    }
    if modifiers.contains(Modifier::RAPID_BLINK) {
        execute!(writer, SetAttribute(Attribute::RapidBlink))?;
    }
    if modifiers.contains(Modifier::REVERSED) {
        execute!(writer, SetAttribute(Attribute::Reverse))?;
    }
    if modifiers.contains(Modifier::HIDDEN) {
        execute!(writer, SetAttribute(Attribute::Hidden))?;
    }
    if modifiers.contains(Modifier::CROSSED_OUT) {
        execute!(writer, SetAttribute(Attribute::CrossedOut))?;
    }
    Ok(())
}

fn stdout_supports_color() -> bool {
    let force = force_color_enabled();
    if force {
        force_color_output(true);
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    io::stdout().is_terminal()
}

fn stderr_supports_color() -> bool {
    let force = force_color_enabled();
    if force {
        force_color_output(true);
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    io::stderr().is_terminal()
}

fn force_color_enabled() -> bool {
    is_force_color_var_set("AGX_FORCE_COLOR") || is_force_color_var_set("CLICOLOR_FORCE")
}

fn is_force_color_var_set(name: &str) -> bool {
    match std::env::var(name) {
        Ok(value) => value != "0",
        Err(_) => false,
    }
}

fn style_for(kind: MessageKind) -> Style {
    match kind {
        MessageKind::Path => Style::new().fg(Color::Cyan).add_modifier(Modifier::DIM),
        MessageKind::Log => Style::new().fg(Color::Blue).add_modifier(Modifier::DIM),
        MessageKind::Hint => Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        MessageKind::Quote => Style::new()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::ITALIC),
        MessageKind::Warning => Style::new()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD),
        MessageKind::Error => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
    }
}
