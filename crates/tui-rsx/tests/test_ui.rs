use leptos_reactive::{create_runtime, create_scope, Scope};
use ratatui::{
    backend::{Backend, TestBackend},
    buffer::Buffer,
    style::{Color, Style},
    Terminal,
};
use tui_rsx::{prelude::*, view};

#[test]
fn standalone_widget() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn widget_no_props() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn simple_column() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn conditional() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let a = 1;
        let mut view = view! { cx,
            <Column>
                {
                    match a {
                        1 =>  view!(cx, <block title="test" borders=Borders::ALL/>),
                        _ => view!(cx, <block title="test2" borders=Borders::ALL/>)
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
    })
    .dispose();
}

#[test]
fn list_basic() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn stateful() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = ListState::default();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn list_styled() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn block_children() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn single_child_as_vec() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn single_nested_child_as_vec() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn complex_block_children() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn macro_as_prop() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn array_as_variable() {
    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(15, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let tab_items = vec!["tab1", "tab2"];
        let mut view = view! { cx,
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
    })
    .dispose();
}

#[test]
fn simple_custom_component() {
    #[component]
    fn viewer<B: Backend>(cx: Scope, #[prop(into)] text: String) -> impl View<B> {
        view! { cx,
            <list>
                <>
                    <listItem>{text}</listItem>
                </>
            </list>
        }
    }

    create_scope(create_runtime(), |cx| {
        let backend = TestBackend::new(2, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut view = view! { cx,
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
    })
    .dispose();
}
