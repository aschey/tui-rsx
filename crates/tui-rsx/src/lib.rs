pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, widgets::*, Frame};
    pub use tui_rsx_macros::*;
}

use prelude::*;
use ratatui::{backend::Backend, layout::Rect, Frame};

pub type BlockProps<'a> = Block<'a>;

pub fn block<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: BlockProps) {
    frame.render_widget(props, rect);
}
