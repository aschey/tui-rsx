use ratatui::{backend::TestBackend, Terminal};
use tui_rsx::prelude::*;

pub fn main() {
    let backend = TestBackend::new(32, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = rsx! {
                <Column>
                    <block title="test"/>
                </Column>
            };
            view(f, f.size());
        })
        .unwrap();
}
