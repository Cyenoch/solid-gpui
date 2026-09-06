use crate::native::{Event, NativeChildren, NativeView, ViewCommand};
use gpui::{AppContext, Context, Entity, IntoElement, Render, Window};
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RichTextFormat {
    #[default]
    Markdown,
    Html,
}
impl From<RichTextFormat> for gpui_base::text::TextViewFormat {
    fn from(v: RichTextFormat) -> Self {
        match v {
            RichTextFormat::Markdown => Self::Markdown,
            RichTextFormat::Html => Self::Html,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextSelectionFormat {
    #[default]
    Plain,
    Source,
}
impl From<TextSelectionFormat> for gpui_component::text::SelectionFormat {
    fn from(v: TextSelectionFormat) -> Self {
        match v {
            TextSelectionFormat::Plain => Self::Plain,
            TextSelectionFormat::Source => Self::Source,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TextViewProps {
    pub text: String,
    pub format: RichTextFormat,
    pub selectable: bool,
    pub scrollable: bool,
    pub selection_format: TextSelectionFormat,
    pub max_lines: Option<usize>,
    pub mdx: bool,
}
pub struct TextView {
    state: Entity<gpui_component::text::TextViewState>,
    props: TextViewProps,
    event: Event<String>,
}
#[crate::component]
impl NativeView for TextView {
    type Props = TextViewProps;
    type Event = String;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.max_lines.is_some_and(|n| n == 0 || n > 100000) {
            return Err("maxLines must be 1..100000".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<String>,
        _: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| match props.format {
            RichTextFormat::Markdown => {
                gpui_component::text::TextViewState::markdown(&props.text, cx)
            }
            RichTextFormat::Html => gpui_component::text::TextViewState::html(&props.text, cx),
        });
        Self {
            state,
            props,
            event,
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut Window, cx: &mut Context<Self>) {
        if p.text != self.props.text || p.format != self.props.format {
            self.state.update(cx, |s, cx| {
                if p.format == self.props.format && p.text.starts_with(&self.props.text) {
                    s.push_str(&p.text[self.props.text.len()..], cx);
                } else {
                    s.set_content(p.format.into(), &p.text, cx);
                }
            });
        }
        self.props = p;
    }
    fn event_name() -> &'static str {
        "linkClick"
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("getSelectedText", |this, (): (), _, cx| {
                Ok(this.state.read(cx).selected_text())
            }),
            ViewCommand::new("selectAll", |this, (): (), _, cx| {
                this.state.update(cx, |s, cx| s.select_all(cx));
                Ok(())
            }),
            ViewCommand::new("clearSelection", |this, (): (), _, cx| {
                this.state.update(cx, |s, cx| s.clear_selection(cx));
                Ok(())
            }),
        ]
    }
}
impl Render for TextView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::text::TextView::new(&self.state)
            .selectable(self.props.selectable)
            .scrollable(self.props.scrollable)
            .selection_format(self.props.selection_format.into());
        if let Some(lines) = self.props.max_lines {
            v = v.max_lines(lines);
        }
        if self.props.mdx {
            v = v.markdown_mdx();
        }
        if self.event.is_subscribed() {
            let weak = cx.entity().downgrade();
            v = v.on_link_click(move |url, _, _, cx| {
                let _ = weak.update(cx, |this, _| this.event.emit(url.to_string()));
            });
        }
        v
    }
}
pub(crate) fn definition() -> crate::native::ComponentDefinition {
    __native_component_TextView()
}
