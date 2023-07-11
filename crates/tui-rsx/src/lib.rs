use leptos_reactive::Scope;
use prelude::*;
use ratatui::{backend::Backend, layout::Rect, style::Style, Frame};
pub use tui_rsx_macros::*;
pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};
    pub use tui_rsx_macros::*;
}

macro_rules! impl_widget {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props<'a> = $widget<'a>;

        impl<'a> MakeBuilder for $props<'a> {}

        pub fn $name<B: Backend>(
            #[cfg(feature = "reactive")] _cx: Scope,
            props: $props,
        ) -> impl View<B> + '_ {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(&props, rect)
        }
    };
}

macro_rules! impl_stateful_widget {
    ($name:ident, $widget:ident, $props:ident, $state:ident) => {
        pub type $props<'a> = $widget<'a>;

        pub fn $name<'a, B: Backend>(
            #[cfg(feature = "reactive")] _cx: Scope,
            props: $props<'a>,
            state: &'a mut $state,
        ) -> impl View<B> + 'a {
            move |frame: &mut Frame<B>, rect: Rect| {
                frame.render_stateful_widget(&props, rect, state);
            }
        }
    };
}

// pub trait Props {
//     type Builder;
//     fn builder() -> Self::Builder;
// }

pub trait BuilderFacade {
    fn builder() -> Self;
}

pub trait BuildFacade {
    fn build(self) -> Self;
}

pub trait MakeBuilder {}

impl<T> BuilderFacade for T
where
    T: MakeBuilder + Default,
{
    fn builder() -> Self {
        Self::default()
    }
}

impl<T> BuildFacade for T
where
    T: MakeBuilder,
{
    fn build(self) -> Self {
        self
    }
}

impl<'a> MakeBuilder for Row<'a> {}
impl<'a> MakeBuilder for Cell<'a> {}
impl<'a> MakeBuilder for Span<'a> {}
impl<'a> MakeBuilder for ListItem<'a> {}
impl<'a> MakeBuilder for Line<'a> {}
impl MakeBuilder for Style {}

impl_widget!(block, Block, BlockProps);
impl_widget!(paragraph, Paragraph, ParagraphProps);
impl_widget!(list, List, ListProps);
impl_widget!(tabs, Tabs, TabsProps);
impl_widget!(table, Table, TableProps);
impl_stateful_widget!(stateful_list, List, StatefulListProps, ListState);
impl_stateful_widget!(stateful_table, Table, StatefulTableProps, TableState);

pub trait View<B: Backend> {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
}

impl<B: Backend, F> View<B> for F
where
    F: FnMut(&mut Frame<B>, Rect),
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)(frame, rect)
    }
}

pub trait NewExt<'a, T>
where
    Self: 'a,
{
    fn new(source: T) -> Self;
}

pub trait NewFrom {}

impl<'a, S, T> NewExt<'a, T> for S
where
    S: NewFrom + 'a,
    Self: From<T>,
{
    fn new(source: T) -> Self {
        Self::from(source)
    }
}

impl<'a> NewFrom for Line<'a> {}
impl<'a> NewFrom for Span<'a> {}
impl<'a> NewFrom for Cell<'a> {}
impl<'a> NewFrom for Text<'a> {}

pub trait StyleExt<'a> {
    fn style(self, style: Style) -> Self;
}

impl<'a> StyleExt<'a> for Span<'a> {
    fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> StyleExt<'a> for Text<'a> {
    fn style(mut self, style: Style) -> Self {
        self.reset_style();
        self.patch_style(style);
        self
    }
}
