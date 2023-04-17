#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use ratatui::{backend::TestBackend, Terminal};
use tui_rsx::prelude::*;
pub fn main() {
    let backend = TestBackend::new(32, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            let view = ();
            view(f, f.size());
        })
        .unwrap();
}
