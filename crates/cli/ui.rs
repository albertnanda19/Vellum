use crate::style::Style;

pub struct Ui {
    style: Style,
    width: usize,
}

impl Ui {
    pub fn new(style: Style) -> Self {
        Self { style, width: 40 }
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn rule(&self) -> String {
        "-".repeat(self.width)
    }

    pub fn header(&self, title: &str) -> Vec<String> {
        vec![self.rule(), title.to_string()]
    }

    pub fn footer(&self) -> String {
        self.rule()
    }

    pub fn kv(&self, key: &str, value: &str) -> String {
        let key_pad = 18usize;
        format!("{key:<key_pad$}: {value}")
    }

    pub fn ok_line(&self, message: &str) -> String {
        format!("{} {}", self.style.ok(), message)
    }

    pub fn info_line(&self, message: &str) -> String {
        format!("{} {}", self.style.arrow(), message)
    }

    pub fn list_item(&self, label: &str, status: &str) -> String {
        self.list_item_with_suffix(label, status, None)
    }

    pub fn list_item_with_suffix(&self, label: &str, status: &str, suffix: Option<&str>) -> String {
        let label_width = 30usize;
        let status_width = 2usize;
        let dots = if label.len() >= label_width {
            String::new()
        } else {
            ".".repeat(label_width - label.len())
        };

        match suffix {
            Some(suffix) if !suffix.is_empty() => format!(
                "  {} {label}{dots}{status:>status_width$} {suffix}",
                self.style.bullet()
            ),
            _ => format!(
                "  {} {label}{dots}{status:>status_width$}",
                self.style.bullet()
            ),
        }
    }
}
