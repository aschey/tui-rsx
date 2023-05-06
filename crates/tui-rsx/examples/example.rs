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
            let view = view! {
            move <Row>
                    <Column percentage=50>
                        <tabs select=0 block=prop!{ <block borders=Borders::ALL/> }>
                            <spans>"test"</spans>
                            <spans>
                                <span style=prop!{ <style fg=Color::Green/> }>"test3"</span>
                                {Span::from("test4")}
                            </spans>
                        </tabs>
                    </Column>
                    <Column percentage=50>
                        <list>
                            <listItem>"test3"</listItem>
                            <listItem>"test4"</listItem>
                        </list>
                    </Column>
                </Row>
            };

            view(f, f.size());
        })
        .unwrap();

    disable_raw_mode()?;
    println!();
    Ok(())
}
