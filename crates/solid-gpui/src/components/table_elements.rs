//! Native compound table elements preserve size and index propagation.
use super::ControlSize;
use gpui::{App, IntoElement, RenderOnce, StyleRefinement, Styled, Window};
use gpui_component::{ChildElement, Sizable, Size, table};

#[derive(IntoElement)]
enum TablePart {
    Table(table::Table),
    TableHeader(table::TableHeader),
    TableBody(table::TableBody),
    TableFooter(table::TableFooter),
    TableRow(table::TableRow),
    TableHead(table::TableHead),
    TableCell(table::TableCell),
    TableCaption(table::TableCaption),
}
impl RenderOnce for TablePart {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            Self::Table(element) => element.into_any_element(),
            Self::TableHeader(element) => element.into_any_element(),
            Self::TableBody(element) => element.into_any_element(),
            Self::TableFooter(element) => element.into_any_element(),
            Self::TableRow(element) => element.into_any_element(),
            Self::TableHead(element) => element.into_any_element(),
            Self::TableCell(element) => element.into_any_element(),
            Self::TableCaption(element) => element.into_any_element(),
        }
    }
}
impl Styled for TablePart {
    fn style(&mut self) -> &mut StyleRefinement {
        match self {
            Self::Table(element) => element.style(),
            Self::TableHeader(element) => element.style(),
            Self::TableBody(element) => element.style(),
            Self::TableFooter(element) => element.style(),
            Self::TableRow(element) => element.style(),
            Self::TableHead(element) => element.style(),
            Self::TableCell(element) => element.style(),
            Self::TableCaption(element) => element.style(),
        }
    }
}
impl Sizable for TablePart {
    fn with_size(self, size: impl Into<Size>) -> Self {
        let size = size.into();
        match self {
            Self::Table(element) => Self::Table(element.with_size(size)),
            Self::TableHeader(element) => Self::TableHeader(element.with_size(size)),
            Self::TableBody(element) => Self::TableBody(element.with_size(size)),
            Self::TableFooter(element) => Self::TableFooter(element.with_size(size)),
            Self::TableRow(element) => Self::TableRow(element.with_size(size)),
            Self::TableHead(element) => Self::TableHead(element.with_size(size)),
            Self::TableCell(element) => Self::TableCell(element.with_size(size)),
            Self::TableCaption(element) => Self::TableCaption(element.with_size(size)),
        }
    }
}
impl ChildElement for TablePart {
    fn with_ix(self, index: usize) -> Self {
        match self {
            Self::Table(element) => Self::Table(element.with_ix(index)),
            Self::TableHeader(element) => Self::TableHeader(element.with_ix(index)),
            Self::TableBody(element) => Self::TableBody(element.with_ix(index)),
            Self::TableFooter(element) => Self::TableFooter(element.with_ix(index)),
            Self::TableRow(element) => Self::TableRow(element.with_ix(index)),
            Self::TableHead(element) => Self::TableHead(element.with_ix(index)),
            Self::TableCell(element) => Self::TableCell(element.with_ix(index)),
            Self::TableCaption(element) => Self::TableCaption(element.with_ix(index)),
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, NativeItems};
    use gpui::{ParentElement, prelude::FluentBuilder};
    #[component]
    fn table(
        items: NativeItems<TablePart>,
        #[prop(default)] size: ControlSize,
        #[prop(default)] accessibility_label: Option<String>,
    ) -> impl IntoElement + Styled {
        TablePart::Table(
            table::Table::new()
                .with_size(size)
                .when_some(accessibility_label, |v, label| v.accessibility_label(label))
                .children(items),
        )
    }
    #[component]
    fn table_header(
        items: NativeItems<TablePart>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        TablePart::TableHeader(table::TableHeader::new().with_size(size).children(items))
    }
    #[component]
    fn table_body(
        items: NativeItems<TablePart>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        TablePart::TableBody(table::TableBody::new().with_size(size).children(items))
    }
    #[component]
    fn table_footer(
        items: NativeItems<TablePart>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        TablePart::TableFooter(table::TableFooter::new().with_size(size).children(items))
    }
    #[component]
    fn table_row(
        items: NativeItems<TablePart>,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement + Styled {
        TablePart::TableRow(table::TableRow::new().with_size(size).children(items))
    }
    #[component]
    fn table_head(
        #[prop(default = 1)] col_span: u16,
        #[prop(default)] align: CellAlign,
        #[prop(default)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        TablePart::TableHead(
            table::TableHead::new()
                .with_size(size)
                .col_span(col_span as usize)
                .when(align == CellAlign::Center, |v| v.text_center())
                .when(align == CellAlign::Right, |v| v.text_right())
                .children(cx.children()),
        )
    }
    #[component]
    fn table_cell(
        #[prop(default = 1)] col_span: u16,
        #[prop(default)] align: CellAlign,
        #[prop(default)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        TablePart::TableCell(
            table::TableCell::new()
                .with_size(size)
                .col_span(col_span as usize)
                .when(align == CellAlign::Center, |v| v.text_center())
                .when(align == CellAlign::Right, |v| v.text_right())
                .children(cx.children()),
        )
    }
    #[component]
    fn table_caption(
        #[prop(default)] size: ControlSize,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        TablePart::TableCaption(
            table::TableCaption::new()
                .with_size(size)
                .children(cx.children()),
        )
    }
}
pub(super) use exports::native_module;
