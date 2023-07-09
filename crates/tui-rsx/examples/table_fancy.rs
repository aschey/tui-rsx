use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use leptos_reactive::{create_runtime, create_scope, Scope};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    Terminal, TerminalOptions, Viewport,
};
use std::io;
use tui_rsx::prelude::*;

pub fn main() {
    create_scope(create_runtime(), run).dispose();
}

pub fn run(cx: Scope) {
    enable_raw_mode().unwrap();

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )
    .unwrap();

    let view = view! { cx,
        <table
            style=prop!(<style fg=Color::White/>)
            header=prop!(
                <row bottom_margin=1 style=prop!(<style fg=Color::Yellow/>)>
                    "Col1" "Col2" "Col3"
                </row>)
            block=prop!(<block title="Table"/>)
            widths=&[Constraint::Length(5), Constraint::Length(5), Constraint::Length(10)]
            column_spacing=1
            highlight_style=prop!(<style add_modifier=Modifier::BOLD/>)
            highlight_symbol=">>"
        >
            <row>"Row11" "Row12"</row>
            <row style=prop!(<style fg=Color::Blue/>)>"Row21" "Row22"</row>
            <row>
                <cell>"Row31"</cell>
                <cell style=prop!(<style fg=Color::Yellow/>)>"Row32"</cell>
                <cell>
                    <line>
                        <span>"Row"</span>
                        <span style=prop!(<style fg=Color::Green/>)>"33"</span>
                    </line>
                </cell>
            </row>
            <row height=2>
                <cell>"Row\n41"</cell>
                <cell>"Row\n42"</cell>
                <cell>"Row\n43"</cell>
            </row>
        </table>
    };

    terminal
        .draw(|f| {
            view(f, f.size());
        })
        .unwrap();

    disable_raw_mode().unwrap();
    println!();
}
