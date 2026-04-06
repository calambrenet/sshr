use nu_ansi_term::{Color, Style};
use std::sync::atomic::{AtomicBool, Ordering};

static COLORS_ENABLED: AtomicBool = AtomicBool::new(true);

/*
pub fn disable_colors() {
    COLORS_ENABLED.store(false, Ordering::Relaxed);
}
*/

pub fn colors_enabled() -> bool {
    COLORS_ENABLED.load(Ordering::Relaxed)
}

/// Aplica estilo solo si los colores están habilitados
fn styled(text: &str, style: Style) -> String {
    if colors_enabled() {
        style.paint(text).to_string()
    } else {
        text.to_string()
    }
}

// Funciones semánticas de color
pub fn host_name(text: &str) -> String {
    styled(text, Color::Cyan.bold())
}

pub fn hostname_addr(text: &str) -> String {
    styled(text, Color::Green.normal())
}

pub fn port_number(text: &str) -> String {
    styled(text, Color::Yellow.normal())
}

pub fn user_name(text: &str) -> String {
    styled(text, Color::Blue.normal())
}

pub fn tag(text: &str) -> String {
    styled(text, Style::new().dimmed())
}

pub fn header(text: &str) -> String {
    styled(text, Style::new().bold().underline())
}

//pub fn warning(text: &str) -> String {
//    styled(text, Color::Yellow.bold())
//}

//pub fn error(text: &str) -> String {
//    styled(text, Color::Red.bold())
//}

//pub fn success(text: &str) -> String {
//    styled(text, Color::Green.bold())
//}

pub fn dimmed(text: &str) -> String {
    styled(text, Style::new().dimmed())
}
