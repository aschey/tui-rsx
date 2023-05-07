use prelude::*;
use ratatui::{backend::Backend, layout::Rect, style::Style, Frame};
pub use tui_rsx_macros::*;
pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};
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

pub type StatefulListProps<'a> = List<'a>;

pub fn stateful_list<B: Backend>(
    frame: &mut Frame<B>,
    rect: Rect,
    props: StatefulListProps,
    state: &mut ListState,
) {
    frame.render_stateful_widget(props, rect, state);
}

pub type TabsProps<'a> = Tabs<'a>;

pub fn tabs<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: TabsProps) {
    frame.render_widget(props, rect);
}

pub type TableProps<'a> = Table<'a>;

pub fn table<B: Backend>(frame: &mut Frame<B>, rect: Rect, props: TableProps) {
    frame.render_widget(props, rect);
}

pub type StatefulTableProps<'a> = Table<'a>;

pub fn stateful_table<B: Backend>(
    frame: &mut Frame<B>,
    rect: Rect,
    props: TableProps,
    state: &mut TableState,
) {
    frame.render_stateful_widget(props, rect, state);
}

pub trait NewExt<'a>
where
    Self: 'a,
{
    fn new<T>(source: T) -> Self
    where
        Self: From<T>;
}

pub trait NewFrom {}

impl<'a, S> NewExt<'a> for S
where
    S: NewFrom + 'a,
{
    fn new<T>(source: T) -> Self
    where
        Self: From<T>,
    {
        Self::from(source)
    }
}

impl<'a> NewFrom for Spans<'a> {}
impl<'a> NewFrom for Span<'a> {}
impl<'a> NewFrom for Cell<'a> {}
impl<'a> NewFrom for Text<'a> {}

pub trait SpanExt<'a> {
    fn style(self, style: Style) -> Self;
}

impl<'a> SpanExt<'a> for Span<'a> {
    fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}
