use std::io::IsTerminal;

#[derive(Clone, Copy)]
pub enum Color {
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Copy)]
pub struct Style {
    use_color_stdout: bool,
    use_color_stderr: bool,
}

impl Style {
    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some();

        let use_color_stdout = !no_color && std::io::stdout().is_terminal();
        let use_color_stderr = !no_color && std::io::stderr().is_terminal();

        Self {
            use_color_stdout,
            use_color_stderr,
        }
    }

    pub fn ok(&self) -> String {
        self.paint_stdout(Color::Green, "✔")
    }

    pub fn bullet(&self) -> String {
        "•".to_string()
    }

    pub fn arrow(&self) -> String {
        self.paint_stdout(Color::Yellow, "→")
    }

    pub fn ok_text(&self, text: &str) -> String {
        self.paint_stdout(Color::Green, text)
    }

    pub fn paint_stdout(&self, color: Color, text: &str) -> String {
        if !self.use_color_stdout {
            return text.to_string();
        }
        paint(color, text)
    }

    pub fn paint_stderr(&self, color: Color, text: &str) -> String {
        if !self.use_color_stderr {
            return text.to_string();
        }
        paint(color, text)
    }
}

fn paint(color: Color, text: &str) -> String {
    let code = match color {
        Color::Green => "32",
        Color::Yellow => "33",
        Color::Red => "31",
    };

    format!("\u{1b}[{code}m{text}\u{1b}[0m")
}
