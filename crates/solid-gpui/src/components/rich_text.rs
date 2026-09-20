use super::motion_types::MotionEasing;
use crate::native::{Event, NativeChildren, NativeView, ViewCommand};
use gpui::{AppContext, Context, Entity, IntoElement, Render, Window};
use std::time::Duration;
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
/// Streamed-text fade policy for a text view. `true` adopts the upstream
/// theme timing (each appended chunk reaches full color over 350ms along an
/// ease-out curve); `false` turns a previously enabled fade off; the explicit
/// form sets the full motion policy, including an optional word-by-word
/// stagger that is compressed for long updates. Omitted leaves the retained
/// state's own policy alone.
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum TextStreamFadeConfig {
    Enabled(bool),
    Timing {
        duration_ms: u32,
        #[serde(default)]
        stagger_ms: u32,
        #[serde(default)]
        easing: MotionEasing,
    },
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
    pub frontmatter: bool,
    pub stream_fade: Option<TextStreamFadeConfig>,
}
fn markdown_extensions(props: &TextViewProps) -> gpui_component::text::MarkdownExtensions {
    let mut extensions = gpui_component::text::MarkdownExtensions::default();
    if props.mdx {
        extensions = extensions.mdx();
    }
    if props.frontmatter {
        extensions = extensions
            .frontmatter()
            .plugin(gpui_component::text::FrontmatterPlugin::new());
    }
    extensions
}
/// The upstream theme timing `TextView::stream_fade(true)` projects: each
/// appended chunk reaches full color over upstream's theme duration along an
/// ease-out curve.
fn theme_stream_fade_motion() -> gpui_base::text::TextViewMotion {
    gpui_base::text::TextViewMotion::default()
        .with_stream_fade(gpui_component::text::STREAM_FADE)
        .with_stream_fade_easing(gpui_base::motion::Easing::EaseOut)
}
/// The motion policy a props value installs. `None` deliberately leaves the
/// retained state's own policy alone; explicit disabling is `Enabled(false)`,
/// whose zero durations fade nothing.
fn stream_fade_motion(
    fade: Option<TextStreamFadeConfig>,
) -> Option<gpui_base::text::TextViewMotion> {
    match fade {
        None => None,
        Some(TextStreamFadeConfig::Enabled(false)) => {
            Some(gpui_base::text::TextViewMotion::default())
        }
        Some(TextStreamFadeConfig::Enabled(true)) => Some(theme_stream_fade_motion()),
        Some(TextStreamFadeConfig::Timing {
            duration_ms,
            stagger_ms,
            easing,
        }) => Some(
            gpui_base::text::TextViewMotion::default()
                .with_stream_fade(Duration::from_millis(duration_ms as u64))
                .with_stream_fade_stagger(Duration::from_millis(stagger_ms as u64))
                .with_stream_fade_easing(easing.into()),
        ),
    }
}
pub struct TextView {
    state: Entity<gpui_component::text::TextViewState>,
    props: TextViewProps,
    event: Event<String>,
    markdown_extensions: gpui_component::text::MarkdownExtensions,
}
#[crate::component]
impl NativeView for TextView {
    type Props = TextViewProps;
    type Event = String;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.frontmatter && p.format != RichTextFormat::Markdown {
            return Err("frontmatter requires Markdown format".into());
        }
        if p.max_lines.is_some_and(|n| n == 0 || n > 100000) {
            return Err("maxLines must be 1..100000".into());
        }
        if let Some(TextStreamFadeConfig::Timing {
            duration_ms,
            stagger_ms,
            ..
        }) = p.stream_fade
            && (duration_ms > 60000 || stagger_ms > 60000)
        {
            return Err("stream fade duration and stagger must not exceed 60000 ms".into());
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
        // The initial policy is installed at mount, before any later text
        // update can land, so a first streamed chunk is tracked under the
        // configured policy. Initial content itself is not streamed and shows
        // settled.
        if let Some(motion) = stream_fade_motion(props.stream_fade) {
            state.update(cx, |s, _| s.set_motion(motion));
        }
        let markdown_extensions = markdown_extensions(&props);
        Self {
            markdown_extensions,
            state,
            props,
            event,
        }
    }
    fn update(&mut self, p: Self::Props, _: &mut Window, cx: &mut Context<Self>) {
        // A policy change lands before the text update below, so the chunk
        // appended by the same props update is tracked — and fades — under
        // the new policy even if fading was disabled until now.
        if p.stream_fade != self.props.stream_fade
            && let Some(motion) = stream_fade_motion(p.stream_fade)
        {
            self.state.update(cx, |s, _| s.set_motion(motion));
        }
        if p.text != self.props.text || p.format != self.props.format {
            self.state.update(cx, |s, cx| {
                if p.format == self.props.format && p.text.starts_with(&self.props.text) {
                    s.push_str(&p.text[self.props.text.len()..], cx);
                } else {
                    s.set_content(p.format.into(), &p.text, cx);
                }
            });
        }
        if p.frontmatter != self.props.frontmatter || p.mdx != self.props.mdx {
            self.markdown_extensions = markdown_extensions(&p);
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
        v = v.markdown_extensions(self.markdown_extensions.clone());
        // Re-applied every frame, so the state's policy follows the latest
        // props; `None` keeps deferring to whatever the state already owns.
        if let Some(motion) = stream_fade_motion(self.props.stream_fade) {
            v = v.motion(motion);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn props(json: &str) -> TextViewProps {
        crate::native::decode_json(json.as_bytes()).unwrap()
    }

    #[test]
    fn stream_fade_timing_is_bounded_like_other_motion_timing() {
        assert!(
            TextView::validate_props(&props(r#"{"text":"hi","streamFade":{"durationMs":60001}}"#))
                .is_err()
        );
        assert!(
            TextView::validate_props(&props(
                r#"{"text":"hi","streamFade":{"durationMs":100,"staggerMs":60001}}"#
            ))
            .is_err()
        );
        assert!(
            TextView::validate_props(&props(
                r#"{"text":"hi","streamFade":{"durationMs":350,"staggerMs":30}}"#
            ))
            .is_ok()
        );
        assert!(
            TextView::validate_props(&props(r#"{"text":"hi","streamFade":false}"#)).is_ok(),
            "an explicit disable needs no timing and is always valid"
        );
    }
}
