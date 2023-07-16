use prelude::*;
use ratatui::{backend::Backend, layout::Rect, style::Style, Frame};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};
use typemap::Key;

pub use once_cell;
pub use tui_rsx_macros::*;
pub use typed_builder;
pub use typemap;

pub mod prelude {
    pub use super::*;
    pub use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};
}

macro_rules! impl_widget {
    ($name:ident, $widget:ident, $props:ident) => {
        pub type $props = $widget<'static>;

        impl MakeBuilder for $props {}

        pub fn $name<T, B: Backend + 'static>(_cx: T, props: $props) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| frame.render_widget(&props, rect)
        }
    };
}

macro_rules! impl_stateful_widget {
    ($name:ident, $name_owned:ident, $widget:ident, $props:ident, $owned_props: ident, $state:ident) => {
        pub type $props = $widget<'static>;

        pub fn $name<T, B: Backend + 'static>(
            _cx: T,
            props: $props,
            state: &'static mut $state,
        ) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| {
                frame.render_stateful_widget(&props, rect, state);
            }
        }

        pub type $owned_props = $widget<'static>;

        pub fn $name_owned<T, B: Backend + 'static>(
            _cx: T,
            props: $props,
            mut state: $state,
        ) -> impl View<B> {
            move |frame: &mut Frame<B>, rect: Rect| {
                frame.render_stateful_widget(&props, rect, &mut state);
            }
        }
    };
}

pub struct KeyWrapper<T>(PhantomData<T>);

impl<B: Backend + 'static> Key for KeyWrapper<B> {
    type Value = HashMap<u32, Rc<RefCell<dyn View<B>>>>;
}

pub trait BuilderFacade {
    fn builder() -> Self;
}

pub trait BuildFacade {
    fn build(self) -> Self;
    fn __caller_id(self, caller_id: u32) -> Self;
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

    fn __caller_id(self, _caller_id: u32) -> Self {
        self
    }
}

impl<'a> MakeBuilder for Row<'a> {}
impl<'a> MakeBuilder for Cell<'a> {}
impl<'a> MakeBuilder for Span<'a> {}
impl<'a> MakeBuilder for ListItem<'a> {}
impl<'a> MakeBuilder for Line<'a> {}
impl<'a> MakeBuilder for Text<'a> {}
impl MakeBuilder for Style {}
impl MakeBuilder for ListState {}
impl MakeBuilder for TableState {}

impl_widget!(block, Block, BlockProps);
impl_widget!(paragraph, Paragraph, ParagraphProps);
impl_widget!(list, List, ListProps);
impl_widget!(tabs, Tabs, TabsProps);
impl_widget!(table, Table, TableProps);
impl_stateful_widget!(
    stateful_list,
    stateful_list_owned,
    List,
    StatefulListProps,
    StatefulListOwnedProps,
    ListState
);
impl_stateful_widget!(
    stateful_table,
    stateful_table_owned,
    Table,
    StatefulTableProps,
    StatefulTableOwnedProps,
    TableState
);

pub trait View<B: Backend> {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
    fn into_boxed_view(self) -> Box<dyn View<B>>;
}

impl<B, F> View<B> for F
where
    B: Backend + 'static,
    F: FnMut(&mut Frame<B>, Rect) + 'static,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

impl<B> View<B> for Box<dyn View<B>>
where
    B: Backend + 'static,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (**self).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        self
    }
}

impl<B: Backend + 'static> View<B> for Rc<RefCell<dyn View<B>>> {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        self.borrow_mut().view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
    }
}

pub trait LazyView<B: Backend> {
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect);
}

impl<B: Backend, F, Ret> LazyView<B> for F
where
    F: FnMut() -> Ret,
    Ret: View<B>,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self)().view(frame, rect)
    }
}

pub struct LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    f: F,
    _phantom: PhantomData<B>,
}

impl<B, F> LazyViewWrapper<B, F>
where
    B: Backend,
    F: LazyView<B>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<B, F> View<B> for LazyViewWrapper<B, F>
where
    B: Backend + 'static,
    F: LazyView<B> + 'static,
{
    fn view(&mut self, frame: &mut Frame<B>, rect: Rect) {
        (self.f).view(frame, rect)
    }

    fn into_boxed_view(self) -> Box<dyn View<B>> {
        Box::new(self)
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

pub trait IntoBoxed<T: ?Sized> {
    fn into_boxed(self) -> Box<T>;
}

impl<F, R> IntoBoxed<dyn Fn() -> R> for F
where
    F: Fn() -> R + 'static,
{
    fn into_boxed(self: F) -> Box<dyn Fn() -> R> {
        Box::new(self)
    }
}
