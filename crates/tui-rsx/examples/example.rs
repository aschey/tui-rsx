use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use leptos_reactive::{create_runtime, create_scope, Scope};
use ratatui::{backend::Backend, backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};
use std::io;
use tui_rsx::prelude::*;
use typed_builder::TypedBuilder;

pub fn main() {
    create_scope(create_runtime(), run).dispose();
}

fn run(cx: Scope) {
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
        <Column>
            <block default length=4/>
            <counter count=0/>
            <Viewer text="blah".to_string()/>

        </Column>
    };

    terminal
        .draw(|f| {
            view(f, f.size());
        })
        .unwrap();

    disable_raw_mode().unwrap();
    println!();
}

#[derive(TypedBuilder)]
pub struct CounterProps {
    count: usize,
}

fn counter<B: Backend>(cx: Scope, props: CounterProps) -> impl Fn(&mut Frame<B>, Rect) {
    let CounterProps { count } = props;
    view! { cx,
        <block title=format!("count {count}")/>
    }
}

#[component]
fn viewer<B: Backend>(
    cx: Scope,
    text: String,
    #[prop(default = 20)] blah: usize,
) -> impl Fn(&mut Frame<B>, Rect) {
    view! { cx,
        <list>
            <listItem>{text}</listItem>
            <listItem>"test2"</listItem>
        </list>
    }
}
