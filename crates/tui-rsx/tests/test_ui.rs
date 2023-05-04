use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    style::{Color, Style},
    Terminal,
};
use tui_rsx::prelude::*;
use tui_rsx_macros::rsx;

#[test]
fn standalone_widget() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let view = rsx! {
                <block title="test" borders=Borders::ALL/>
            };
            view(f, f.size());
        })
        .unwrap();

    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn widget_no_props() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let view = rsx! {
                <Column>
                    <block default/>
                </Column>
            };
            view(f, f.size());
        })
        .unwrap();

    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "          ",
        "          ",
        "          ",
    ]));
}

#[test]
fn simple_column() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let view = rsx! {
                <Column>
                    <block title="test" borders=Borders::ALL/>
                </Column>
            };
            view(f, f.size());
        })
        .unwrap();

    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌test────┐",
        "│        │",
        "└────────┘",
    ]));
}

#[test]
fn test_list_basic() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = rsx! {
                <Column>
                    <list>
                        <listItem>"test1"</listItem>
                        <listItem>"test2"</listItem>
                    </list>
                </Column>
            };
            view(f, f.size());
        })
        .unwrap();

    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "test1     ",
        "test2     ",
        "          ",
    ]));
}

#[test]
fn test_list_styled() {
    let backend = TestBackend::new(15, 3);
    let mut terminal = Terminal::new(backend).unwrap();
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

    let mut expected = Buffer::with_lines(vec![
        "test1          ",
        "test2          ",
        "               ",
    ]);

    for x in 0..15 {
        expected.get_mut(x, 0).set_fg(Color::Black);
    }

    terminal.backend().assert_buffer(&expected);
}
