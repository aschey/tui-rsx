use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    style::{Color, Style},
    Terminal,
};
use tui_rsx::{prelude::*, view};

#[test]
fn standalone_widget() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let view = view! {
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
            let view = view! {
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
            let view = view! {
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
            let view = view! {
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
fn list_styled() {
    let backend = TestBackend::new(15, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = view! {
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

#[test]
fn block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = view! {
                <Column>
                    <tabs>
                        {"tab1".into()}
                        {"tab2".into()}
                    </tabs>
                </Column>
            };

            view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn single_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = view! {
                <Column>
                    <tabs>
                        <>{"tab1".into()}</>
                    </tabs>
                </Column>
            };

            view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
}

#[test]
fn complex_block_children() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = view! {
                <Column>
                    <tabs select=0>
                        <spans>"tab1"</spans>
                        <spans>{vec![Span::from("tab2")]}</spans>
                    </tabs>
                </Column>
            };

            view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn macro_as_prop() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = view! {
                <Column>
                    <paragraph block=prop!{<block borders=Borders::ALL/>}>
                        "test"
                    </paragraph>
                </Column>
            };

            view(f, f.size());
        })
        .unwrap();
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌────────┐",
        "│test    │",
        "└────────┘",
    ]));
}
