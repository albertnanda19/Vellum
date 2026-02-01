use crate::error::CliError;
use crate::output;
use crate::style::{Color, Style};
use crate::ui::Ui;

pub fn print(err: &CliError) {
    let style = Style::detect();
    let ui = Ui::new(style);

    output::error(ui.rule());
    output::error("Vellum Error");
    output::error(ui.rule());

    let title = err.title();
    output::error(style.paint_stderr(Color::Red, title));
    output::error("");

    if let Some(reason) = err.reason() {
        output::error("Reason:");
        output::error(reason);
        output::error("");
    }

    if let Some(meaning) = err.meaning() {
        output::error("What this means:");
        output::error(meaning);
        output::error("");
    }

    if let Some(action) = err.action() {
        output::error("Suggested action:");
        output::error(action);
        output::error("");
    }

    output::error(ui.rule());
}
