pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, widgets::*, Frame};
    pub use tui_rsx_macros::*;
}

use prelude::*;
use ratatui::{backend::Backend, layout::Rect, text::Text, Frame};

pub trait CustomDefault {
    fn default() -> Self;
}

pub type BlockProps<'a> = Block<'a>;

pub fn block<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: BlockProps) {
    frame.render_widget(props, rect);
}

pub type ParagraphProps<'a> = Paragraph<'a>;

pub trait ParagraphPropsExt<'a>
where
    Self: 'a,
{
    fn text(self, text: impl Into<Text<'a>>) -> Self;
}

impl<'a> ParagraphPropsExt<'a> for Paragraph<'a> {
    fn text(self, text: impl Into<Text<'a>>) -> Self {
        Self::new(text)
    }
}

impl<'a> CustomDefault for Paragraph<'a> {
    fn default() -> Self {
        Self::new(Text::default())
    }
}

pub fn paragraph<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: ParagraphProps) {
    frame.render_widget(props, rect);
}
