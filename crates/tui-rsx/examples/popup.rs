use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use tui_rsx::{
    components::{Popup, PopupProps},
    mount,
    prelude::*,
};

#[derive(Clone)]
struct App {
    show_popup: bool,
}

impl App {
    fn new() -> App {
        App { show_popup: false }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend + 'static>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => app.show_popup = !app.show_popup,
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend + 'static>(f: &mut Frame<B>, app: &App) {
    let text = if app.show_popup {
        "Press p to close the popup"
    } else {
        "Press p to show the popup"
    };
    let app = app.clone();
    let mut view = mount! {
        <overlay>
            <column>
                <paragraph percentage=20 alignment=Alignment::Center wrap=prop!(<wrap trim=true/>)>
                    {text.slow_blink()}
                </paragraph>
                <block percentage=80 title="Content" borders=Borders::ALL on_blue/>
            </column>
            {
                if app.show_popup {
                    view! {
                        <Popup percent_x=60 percent_y=20>
                            {move || view!(<block title="Popup" borders=Borders::ALL/>)}
                        </Popup>
                    }.into_boxed_view()
                } else {
                    view! {
                        <column/>
                    }.into_boxed_view()
                }
            }
        </overlay>
    };
    view.view(f, f.size());
}
