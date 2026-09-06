//! Native content, message and attachment composition.
use super::ControlSize;
use super::primitives::Orientation;
use super::validation::{DescriptionColumns, GridSpan, LogicalPixels};
use gpui::{App, IntoElement, RenderOnce, Window};
use gpui_component::description_list::DescriptionItem as NativeDescriptionItem;

#[derive(IntoElement)]
struct NativeOnce<T: RenderOnce + 'static>(T);
impl<T: RenderOnce + 'static> RenderOnce for NativeOnce<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(window, cx)
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageAlignment {
    #[default]
    Start,
    End,
}
impl From<MessageAlignment> for gpui_component::message::MessageAlignment {
    fn from(value: MessageAlignment) -> Self {
        match value {
            MessageAlignment::Start => Self::Start,
            MessageAlignment::End => Self::End,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BubbleVariant {
    #[default]
    Filled,
    Secondary,
    Muted,
    Tinted,
    Outline,
    Ghost,
    Destructive,
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentStatus {
    #[default]
    Complete,
    Pending,
    Uploading,
    Processing,
    Failed,
}
impl From<AttachmentStatus> for gpui_component::attachment::AttachmentStatus {
    fn from(value: AttachmentStatus) -> Self {
        match value {
            AttachmentStatus::Complete => Self::Complete,
            AttachmentStatus::Pending => Self::Pending,
            AttachmentStatus::Uploading => Self::Uploading,
            AttachmentStatus::Processing => Self::Processing,
            AttachmentStatus::Failed => Self::Failed,
        }
    }
}

#[crate::native_module(name = "gpui-component")]
mod exports {
    use super::*;
    use crate::native::{ElementContext, Event, NativeItems, NativeSlot};
    use gpui::{ParentElement, Styled, prelude::FluentBuilder, px};
    use gpui_component::{Sizable, attachment, bubble, description_list, message};
    #[component(validate = |p,c| {
        for item in c.native_props::<__NativePropsDescriptionItem>()? {
            if !item.separator && item.span.0 > p.columns.0 { return Err("DescriptionItem.span must not exceed DescriptionList.columns".into()); }
        }
        Ok(())
    })]
    fn description_list(
        items: NativeItems<NativeDescriptionItem>,
        #[prop(default)] orientation: Orientation,
        #[prop(default)] bordered: bool,
        #[prop(default = DescriptionColumns(1))] columns: DescriptionColumns,
        #[prop(default = LogicalPixels(100.0))] label_width: LogicalPixels,
        #[prop(default)] size: ControlSize,
    ) -> impl IntoElement {
        description_list::DescriptionList::new()
            .layout(orientation.into())
            .bordered(bordered)
            .columns(columns.0 as usize)
            .label_width(px(label_width.0))
            .with_size(size)
            .children(items)
    }
    #[component(descriptor)]
    fn description_item(
        #[prop(default = GridSpan(1))] span: GridSpan,
        #[prop(default)] separator: bool,
        label: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl Styled {
        if separator {
            NativeDescriptionItem::separator()
        } else {
            NativeDescriptionItem::new(label.into_any_element())
                .value(cx.content().into_any_element())
                .span(span.0 as usize)
        }
    }
    #[component]
    fn description_text(cx: &mut ElementContext) -> impl IntoElement {
        description_list::DescriptionText::AnyElement(cx.content().into_any_element())
    }
    #[component]
    fn message_group(cx: &mut ElementContext) -> impl IntoElement + Styled {
        message::MessageGroup::new().children(cx.children())
    }
    #[component]
    fn message(
        #[prop(default)] alignment: MessageAlignment,
        avatar: NativeSlot,
        header: NativeSlot,
        footer: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        message::Message::new()
            .alignment(alignment.into())
            .when(!avatar.is_empty(), |v| v.avatar(avatar))
            .when(!header.is_empty(), |v| {
                v.header(message::MessageHeader::new().child(header))
            })
            .content(message::MessageContent::new().children(cx.children()))
            .when(!footer.is_empty(), |v| {
                v.footer(message::MessageFooter::new().child(footer))
            })
    }
    #[component]
    fn message_avatar(cx: &mut ElementContext) -> impl IntoElement + Styled {
        message::MessageAvatar::new().children(cx.children())
    }
    #[component]
    fn message_header(
        #[prop(default)] content_inset: bool,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        message::MessageHeader::new()
            .content_inset(content_inset)
            .children(cx.children())
    }
    #[component]
    fn message_content(cx: &mut ElementContext) -> impl IntoElement + Styled {
        message::MessageContent::new().children(cx.children())
    }
    #[component]
    fn message_footer(
        #[prop(default)] content_inset: bool,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        message::MessageFooter::new()
            .content_inset(content_inset)
            .children(cx.children())
    }
    #[component]
    fn bubble(
        #[prop(default)] alignment: MessageAlignment,
        #[prop(default)] variant: BubbleVariant,
        reactions: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        let variant = match variant {
            BubbleVariant::Filled => bubble::BubbleVariant::Filled,
            BubbleVariant::Secondary => bubble::BubbleVariant::Secondary,
            BubbleVariant::Muted => bubble::BubbleVariant::Muted,
            BubbleVariant::Tinted => bubble::BubbleVariant::Tinted,
            BubbleVariant::Outline => bubble::BubbleVariant::Outline,
            BubbleVariant::Ghost => bubble::BubbleVariant::Ghost,
            BubbleVariant::Destructive => bubble::BubbleVariant::Destructive,
        };
        bubble::Bubble::new()
            .alignment(alignment.into())
            .with_variant(variant)
            .content(bubble::BubbleContent::new().children(cx.children()))
            .when(!reactions.is_empty(), |v| {
                v.reactions(bubble::BubbleReactions::new().child(reactions))
            })
    }
    #[component]
    fn bubble_content(cx: &mut ElementContext) -> impl IntoElement + Styled {
        bubble::BubbleContent::new().children(cx.children())
    }
    #[component]
    fn bubble_group(cx: &mut ElementContext) -> impl IntoElement + Styled {
        bubble::BubbleGroup::new().children(cx.children())
    }
    #[component]
    fn bubble_reactions(
        #[prop(default)] alignment: MessageAlignment,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        bubble::BubbleReactions::new()
            .alignment(alignment.into())
            .children(cx.children())
    }
    #[component(children = false)]
    fn attachment(
        #[prop(default)] status: AttachmentStatus,
        #[prop(default)] orientation: Orientation,
        media: NativeSlot,
        content: NativeSlot,
        actions: NativeSlot,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        attachment::Attachment::new()
            .id(cx.id())
            .status(status.into())
            .axis(orientation.into())
            .when(!media.is_empty(), |v| {
                v.media(attachment::AttachmentMedia::new().child(media))
            })
            .when(!content.is_empty(), |v| {
                v.content(attachment::AttachmentContent::new().child(content))
            })
            .when(!actions.is_empty(), |v| {
                v.actions(attachment::AttachmentActions::new().child(actions))
            })
            .when(on_press.is_subscribed(), |v| {
                v.on_click(move |_, _, _| on_press.emit(()))
            })
    }
    #[component]
    fn attachment_media(
        #[prop(default)] src: Option<String>,
        overlay: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        attachment::AttachmentMedia::new()
            .when_some(src, |v, s| v.src(s))
            .when(!overlay.is_empty(), |v| v.overlay(overlay))
            .children(cx.children())
    }
    #[component]
    fn attachment_content(
        title: NativeSlot,
        description: NativeSlot,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        attachment::AttachmentContent::new()
            .child(title)
            .child(description)
            .children(cx.children())
    }
    #[component(children = false)]
    fn attachment_title(
        text: String,
        #[prop(default)] status: AttachmentStatus,
    ) -> impl IntoElement + Styled {
        attachment::AttachmentTitle::new(text).status(status.into())
    }
    #[component(children = false)]
    fn attachment_description(
        text: String,
        #[prop(default)] status: AttachmentStatus,
    ) -> impl IntoElement + Styled {
        attachment::AttachmentDescription::new(text).status(status.into())
    }
    #[component]
    fn attachment_actions(cx: &mut ElementContext) -> impl IntoElement + Styled {
        attachment::AttachmentActions::new().children(cx.children())
    }
    #[component]
    fn attachment_group(cx: &mut ElementContext) -> impl IntoElement + Styled {
        attachment::AttachmentGroup::new(cx.id()).children(cx.children())
    }
    #[component]
    fn list_item(
        #[prop(default)] selected: bool,
        #[prop(default)] confirmed: bool,
        #[prop(default)] disabled: bool,
        #[prop(default)] separator: bool,
        suffix: NativeSlot,
        on_press: Event<()>,
        cx: &mut ElementContext,
    ) -> impl IntoElement + Styled {
        gpui_component::list::ListItem::new(cx.id())
            .selected(selected)
            .confirmed(confirmed)
            .disabled(disabled)
            .when(separator, |v| v.separator())
            .when(!suffix.is_empty(), |v| v.suffix(move |_, _| suffix.clone()))
            .when(on_press.is_subscribed(), |v| {
                v.on_click(move |_, _, _| on_press.emit(()))
            })
            .children(cx.children())
    }
    #[component]
    fn list_separator_item(cx: &mut ElementContext) -> impl IntoElement {
        NativeOnce(gpui_component::list::ListSeparatorItem::new().children(cx.children()))
    }
    #[component]
    fn title_bar(on_close: Event<()>, cx: &mut ElementContext) -> impl IntoElement + Styled {
        gpui_component::TitleBar::new()
            .when(on_close.is_subscribed(), |v| {
                v.on_close_window(move |_, _, _| on_close.emit(()))
            })
            .children(cx.children())
    }
    #[component]
    fn window_border(
        #[prop(default)] shadow_size: Option<LogicalPixels>,
        #[prop(default)] resize_hit_size: Option<LogicalPixels>,
        cx: &mut ElementContext,
    ) -> impl IntoElement {
        gpui_component::WindowBorder::new()
            .when_some(shadow_size, |v, n| v.shadow_size(px(n.0)))
            .when_some(resize_hit_size, |v, n| v.resize_hit_size(px(n.0)))
            .children(cx.children())
    }
}
pub(super) use exports::native_module;
