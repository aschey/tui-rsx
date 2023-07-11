use ratatui::{
    backend::{Backend, TestBackend},
    buffer::Buffer,
    style::{Color, Style},
    Terminal,
};
use tui_rsx::{prelude::*, view};

#[test]
fn standalone_widget() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut view = view! {
        <block title="test" borders=Borders::ALL/>
    };

    terminal
        .draw(|f| {
            view.view(f, f.size());
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
    let mut view = view! {
        <Column>
            <block default/>
        </Column>
    };
    terminal
        .draw(|f| {
            view.view(f, f.size());
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
    let mut view = view! {
        <Column>
            <block title="test" borders=Borders::ALL/>
        </Column>
    };
    terminal
        .draw(|f| {
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
fn conditional() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let a = 1;
    let mut view = view! {
        <Column>
            {
                match a {
                    1 => Box::new(view!(<block title="test" borders=Borders::ALL/>)),
                    _ => Box::new(view!(<block title="test2" borders=Borders::ALL/>))
                }
            }
        </Column>
    };
    terminal
        .draw(|f| {
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
fn list_basic() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut view = view! {
        <Column>
            <list>
                <listItem>"test1"</listItem>
                <listItem>"test2"</listItem>
            </list>
        </Column>
    };
    terminal
        .draw(|f| {
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
fn stateful() {
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = ListState::default();
    let mut view = view! {
        <stateful_list state=&mut state>
            <listItem>"test1"</listItem>
            <listItem>"test2"</listItem>
        </stateful_list>
    };

    terminal
        .draw(|f| {
            view.view(f, f.size());
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
    let mut view = view! {
        <Column>
            <list>
                <listItem style=Style::default().fg(Color::Black)>"test1"</listItem>
                <listItem>"test2"</listItem>
            </list>
        </Column>
    };
    terminal
        .draw(|f| {
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

    let mut view = view! {
        <Column>
            <tabs>
                "tab1"
                "tab2"
            </tabs>
        </Column>
    };
    terminal
        .draw(|f| {
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
    let mut view = view! {
        <Column>
            <tabs>
                <>{"tab1"}</>
            </tabs>
        </Column>
    };
    terminal
        .draw(|f| {
            view.view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1          "]));
}

#[test]
fn single_nested_child_as_vec() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut view = view! {
        <Column>
            <tabs>
                <>
                    <line>
                        <span>"tab1"</span>
                    </line>
                </>
            </tabs>
        </Column>
    };

    terminal
        .draw(|f| {
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
    let mut view = view! {
        <Column>
            <tabs select=0>
                <line>"tab1"</line>
                <line>{vec![Span::from("tab2")]}</line>
            </tabs>
        </Column>
    };
    terminal
        .draw(|f| {
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
    let mut view = view! {
        <Column>
            <paragraph block=prop!{<block borders=Borders::ALL/>}>
                "test"
            </paragraph>
        </Column>
    };
    terminal
        .draw(|f| {
            view(f, f.size());
        })
        .unwrap();
    terminal.backend().assert_buffer(&Buffer::with_lines(vec![
        "┌────────┐",
        "│test    │",
        "└────────┘",
    ]));
}

#[test]
fn array_as_variable() {
    let backend = TestBackend::new(15, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let tab_items = vec!["tab1", "tab2"];
    let mut view = view! {
        <Column>
            <tabs>
                {tab_items}
            </tabs>
        </Column>
    };
    terminal
        .draw(|f| {
            view.view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec![" tab1 │ tab2   "]));
}

#[test]
fn simple_custom_component() {
    #[component]
    fn viewer<T, B: Backend>(cx: T, #[prop(into)] text: String) -> impl View<B> {
        view! { cx,
            <list>
                <>
                    <listItem>{text}</listItem>
                </>
            </list>
        }
    }

    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut view = view! {
        <Column>
            <Viewer text="hi"/>
        </Column>
    };
    terminal
        .draw(|f| {
            view.view(f, f.size());
        })
        .unwrap();
    terminal
        .backend()
        .assert_buffer(&Buffer::with_lines(vec!["hi"]));
}
