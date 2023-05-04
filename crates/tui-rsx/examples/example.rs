use std::{error::Error, io};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    Terminal, TerminalOptions, Viewport,
};
use tui_rsx::prelude::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )?;

    terminal
        .draw(|f| {
            let view = rsx! {
                <Column>
                    <list>
                        <listItem style=Style::default().fg(Color::Black)>"test1"</listItem>
                        <listItem>"test2"</listItem>
                    </list>
                </Column>
            };
            view(f, f.size());
        })
        .unwrap();

    disable_raw_mode()?;
    println!();
    Ok(())
}
