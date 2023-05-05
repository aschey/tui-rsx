use prelude::*;
use ratatui::{backend::Backend, layout::Rect, Frame};

pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, text::*, widgets::*, Frame};
    pub use tui_rsx_macros::*;
}

pub type BlockProps<'a> = Block<'a>;

pub fn block<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: BlockProps) {
    frame.render_widget(props, rect);
}

pub type ParagraphProps<'a> = Paragraph<'a>;

pub fn paragraph<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: ParagraphProps) {
    frame.render_widget(props, rect);
}

pub type ListProps<'a> = List<'a>;

pub fn list<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: ListProps) {
    frame.render_widget(props, rect);
}

pub fn stateful_list<B: Backend>(
    frame: &mut Frame<B>,
    rect: Rect,
    props: ListProps,
    state: &mut ListState,
) {
    frame.render_stateful_widget(props, rect, state);
}

pub type TabsProps<'a> = Tabs<'a>;

pub fn tabs<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: TabsProps) {
    frame.render_widget(props, rect);
}

pub trait NewSpans<'a> {
    fn new<T>(source: T) -> Self
    where
        Spans<'a>: From<T>;
}

impl<'a> NewSpans<'a> for Spans<'a> {
    fn new<T>(source: T) -> Self
    where
        Spans<'a>: From<T>,
    {
        Self::from(source)
    }
}
