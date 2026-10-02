//! A mounted view owns one bounded CPU frame. No global portable pointer handles.
use crate::native::{ComponentDefinition, Event, NativeChildren, NativeView, ViewCommand};
use gpui::{
    App, AppContext, Context, Entity, Global, IntoElement, ParentElement, Render, RenderImage,
    Styled, Window, px,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const MAX_INLINE_BYTES: usize = 240 * 1024;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const MAX_HOST_BYTES: usize = 64 * 1024 * 1024;
const MAX_DIMENSION: u32 = 4096;

#[derive(Default)]
struct FrameBudget(Arc<AtomicUsize>);
impl Global for FrameBudget {}
struct Lease {
    bytes: usize,
    budget: Arc<AtomicUsize>,
}
impl Lease {
    fn acquire(bytes: usize, cx: &mut App) -> Result<Self, String> {
        if !cx.has_global::<FrameBudget>() {
            cx.set_global(FrameBudget::default());
        }
        let budget = cx.global::<FrameBudget>().0.clone();
        budget
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(bytes)
                    .filter(|next| *next <= MAX_HOST_BYTES)
            })
            .map_err(|_| "live frames exceed the 64 MiB host budget")?;
        Ok(Self { bytes, budget })
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.budget.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}
struct PixelsOwner {
    image: Arc<RenderImage>,
    _lease: Lease,
}
impl PixelsOwner {
    fn new(width: u32, height: u32, mut rgba: Vec<u8>, lease: Lease, cx: &mut App) -> Entity<Self> {
        // GPUI's RenderImage consumes BGRA, whereas the public frame contract is RGBA.
        for pixel in rgba.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
        let pixels = image::RgbaImage::from_raw(width, height, rgba).expect("validated frame size");
        let image = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(
            pixels
        )]));
        cx.new(|cx| {
            cx.on_release(|owner: &mut Self, cx| cx.drop_image(owner.image.clone(), None))
                .detach();
            Self {
                image,
                _lease: lease,
            }
        })
    }
}
fn frame_bytes(width: u32, height: u32) -> Result<usize, String> {
    let bytes = width as u64 * height as u64 * 4;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || bytes > MAX_FRAME_BYTES as u64
    {
        return Err("CPU frames need dimensions from 1 to 4096 and at most 16 MiB of RGBA".into());
    }
    Ok(bytes as usize)
}
#[crate::native_type]
#[derive(Clone, Copy, Debug)]
pub struct FrameUpload {
    pub sequence: u32,
    pub width: u32,
    pub height: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct FrameChunk {
    pub sequence: u32,
    pub offset: usize,
    pub rgba: Vec<u8>,
}
struct Staging {
    upload: FrameUpload,
    bytes: Vec<u8>,
    expected: usize,
    lease: Lease,
}

#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CpuFrame {
    pub sequence: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrameState {
    pub disposed: bool,
    pub sequence: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub retained_bytes: usize,
}
#[derive(Default)]
struct FrameResource {
    current: Option<Entity<PixelsOwner>>,
    staging: Option<Staging>,
    sequence: Option<u32>,
    high_water: Option<u32>,
    dimensions: (u32, u32),
    disposed: bool,
}
impl FrameResource {
    fn admit(&self, sequence: u32) -> Result<(), String> {
        if self.disposed {
            return Err("live frame resource is disposed".into());
        }
        if self.high_water.is_some_and(|previous| sequence <= previous) {
            return Err("frame sequence must increase for this owner".into());
        }
        Ok(())
    }
    fn replace(&mut self, frame: CpuFrame, cx: &mut App) -> Result<FrameState, String> {
        self.admit(frame.sequence)?;
        let bytes = frame_bytes(frame.width, frame.height)?;
        if bytes != frame.rgba.len() || bytes > MAX_INLINE_BYTES {
            return Err("replaceFrame requires exact RGBA bytes, at most 240 KiB; use chunked upload for larger frames".into());
        }
        let lease = Lease::acquire(bytes, cx)?;
        let owner = PixelsOwner::new(frame.width, frame.height, frame.rgba, lease, cx);
        self.current = Some(owner);
        self.dimensions = (frame.width, frame.height);
        self.sequence = Some(frame.sequence);
        self.high_water = Some(frame.sequence);
        self.staging = None;
        Ok(self.state())
    }
    fn begin(&mut self, upload: FrameUpload, cx: &mut App) -> Result<FrameState, String> {
        self.admit(upload.sequence)?;
        if self.staging.is_some() {
            return Err("cancel or present the existing frame upload first".into());
        }
        let expected = frame_bytes(upload.width, upload.height)?;
        let lease = Lease::acquire(expected, cx)?;
        self.staging = Some(Staging {
            upload,
            bytes: Vec::with_capacity(expected),
            expected,
            lease,
        });
        self.high_water = Some(upload.sequence);
        Ok(self.state())
    }
    fn write(&mut self, chunk: FrameChunk) -> Result<FrameState, String> {
        let staging = self.staging.as_mut().ok_or("no active frame upload")?;
        if chunk.sequence != staging.upload.sequence
            || chunk.offset != staging.bytes.len()
            || chunk.rgba.is_empty()
            || chunk.rgba.len() > MAX_INLINE_BYTES
            || chunk.rgba.len() > staging.expected - staging.bytes.len()
        {
            return Err("frame chunks require the active sequence, contiguous offset, and 1..240 KiB within the frame".into());
        }
        staging.bytes.extend_from_slice(&chunk.rgba);
        Ok(self.state())
    }
    fn present(&mut self, sequence: u32, cx: &mut App) -> Result<FrameState, String> {
        let staging = self.staging.as_ref().ok_or("no active frame upload")?;
        if staging.upload.sequence != sequence || staging.bytes.len() != staging.expected {
            return Err("presentFrame requires the complete active frame sequence".into());
        }
        let staging = self.staging.take().unwrap();
        let upload = staging.upload;
        self.current = Some(PixelsOwner::new(
            upload.width,
            upload.height,
            staging.bytes,
            staging.lease,
            cx,
        ));
        self.dimensions = (upload.width, upload.height);
        self.sequence = Some(sequence);
        Ok(self.state())
    }
    fn clear(&mut self) -> Result<FrameState, String> {
        if self.disposed {
            return Err("live frame resource is disposed".into());
        }
        self.current = None;
        self.staging = None;
        self.dimensions = (0, 0);
        Ok(self.state())
    }
    fn dispose(&mut self) -> FrameState {
        self.current = None;
        self.staging = None;
        self.dimensions = (0, 0);
        self.disposed = true;
        self.state()
    }
    fn cancel(&mut self) -> Result<FrameState, String> {
        if self.disposed {
            return Err("live frame resource is disposed".into());
        }
        self.staging = None;
        Ok(self.state())
    }
    fn state(&self) -> FrameState {
        FrameState {
            disposed: self.disposed,
            sequence: self.sequence,
            width: self.dimensions.0,
            height: self.dimensions.1,
            retained_bytes: (self.dimensions.0 as usize * self.dimensions.1 as usize * 4)
                + self.staging.as_ref().map_or(0, |frame| frame.expected),
        }
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum FrameFit {
    #[default]
    Contain,
    Cover,
    Fill,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveFrameProps {
    #[serde(default = "super::recorded_paint::default_height")]
    pub viewport_height: f32,
    #[serde(default)]
    pub fit: FrameFit,
}
struct LiveFrame {
    props: LiveFrameProps,
    resource: FrameResource,
}
impl NativeView for LiveFrame {
    type Props = LiveFrameProps;
    type Event = ();
    fn emits_primary_event() -> bool {
        false
    }
    fn validate_props(props: &Self::Props) -> Result<(), String> {
        super::recorded_paint::dimension(props.viewport_height)
    }
    fn mount(
        props: Self::Props,
        _: Event<()>,
        _: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            props,
            resource: FrameResource::default(),
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, _: &mut Context<Self>) {
        self.props = props;
    }
    fn unmount(&mut self, _: &mut Window, _: &mut App) {
        self.resource.dispose();
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("replaceFrame", Self::replace),
            ViewCommand::new("beginFrame", |this, upload, _, cx| {
                this.resource.begin(upload, cx)
            }),
            ViewCommand::new("writeFrameChunk", |this, chunk, _, _| {
                this.resource.write(chunk)
            }),
            ViewCommand::new("presentFrame", Self::present),
            ViewCommand::new("cancelFrame", |this, (): (), _, _| this.resource.cancel()),
            ViewCommand::new("clear", |this, (): (), _, cx| {
                let state = this.resource.clear()?;
                cx.notify();
                Ok(state)
            }),
            ViewCommand::new("dispose", |this, (): (), _, cx| {
                let state = this.resource.dispose();
                cx.notify();
                Ok(state)
            }),
            ViewCommand::new("getState", |this, (): (), _, _| Ok(this.resource.state())),
        ]
    }
}
impl LiveFrame {
    fn replace(
        &mut self,
        frame: CpuFrame,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<FrameState, String> {
        let state = self.resource.replace(frame, cx)?;
        cx.notify();
        Ok(state)
    }
    fn present(
        &mut self,
        sequence: u32,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<FrameState, String> {
        let state = self.resource.present(sequence, cx)?;
        cx.notify();
        Ok(state)
    }
}
impl Render for LiveFrame {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = self.resource.current.clone();
        let image = owner.as_ref().map(|owner| owner.read(cx).image.clone());
        let fit = self.props.fit;
        gpui::div()
            .w_full()
            .h(px(self.props.viewport_height))
            .overflow_hidden()
            .child(
                gpui::canvas(
                    |_, _, _| (),
                    move |bounds, (), window, _| {
                        let _owner = &owner;
                        let Some(image) = &image else {
                            return;
                        };
                        let pixels = image.size(0);
                        let (w, h) = (pixels.width.0 as f32, pixels.height.0 as f32);
                        let ratio = match fit {
                            FrameFit::Contain => (bounds.size.width.as_f32() / w)
                                .min(bounds.size.height.as_f32() / h),
                            FrameFit::Cover => (bounds.size.width.as_f32() / w)
                                .max(bounds.size.height.as_f32() / h),
                            FrameFit::Fill => 1.,
                        };
                        let dimensions = match fit {
                            FrameFit::Fill => bounds.size,
                            _ => gpui::size(px(w * ratio), px(h * ratio)),
                        };
                        let image_bounds = gpui::Bounds::new(
                            bounds.origin
                                + gpui::point(
                                    (bounds.size.width - dimensions.width) / 2.,
                                    (bounds.size.height - dimensions.height) / 2.,
                                ),
                            dimensions,
                        );
                        if let Err(error) = window.paint_image(
                            bounds,
                            image_bounds,
                            gpui::Corners::default(),
                            image.clone(),
                            0,
                            false,
                        ) {
                            eprintln!("LiveFrame paint failed: {error}");
                        }
                    },
                )
                .size_full(),
            )
    }
}
pub(super) fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<LiveFrame>("LiveFrame")
        .with_semantic_version("1.0.0")
        .with_implementation(include_str!("live_frame.rs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn paint_media_epoch_replacement_releases_pixels_and_rejects_old_owner_commands(
        cx: &mut TestAppContext,
    ) {
        use crate::protocol::{
            Command, CommandMeta, CommandOperation, DecodedMessage, EventPayload, ExtensionField,
            ExtensionProperties, ExtensionValue,
        };
        use crate::{HostProperties, InMemoryAdapter, Node, Snapshot, SolidRoot};
        use std::rc::Rc;
        cx.update(gpui_component::init);
        let module =
            crate::native::ModuleDefinition::new("frame-test", "1.0.0", vec![definition()], vec![]);
        let id = module.id();
        let digest = module.digest();
        let build_digest = module.build_digest();
        let entry = module.component_id("LiveFrame").unwrap();
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(200.), px(100.)), {
            let runtime = runtime.clone();
            move |_, _| SolidRoot::with_extensions(runtime, Rc::new(module))
        });
        let root = window.root(cx).unwrap();
        let snapshot = |epoch| {
            let mut frame = Node::new(2, 1, 0, crate::KIND_EXTENSION);
            frame.host_properties = Some(HostProperties::Extension(ExtensionProperties {
                provider_id: id,
                catalog_digest: digest,
                entry_id: entry,
                entry_version: 1,
                fields: vec![ExtensionField {
                    id: 1,
                    value: ExtensionValue::Bytes(
                        crate::native::encode_native_request(
                            build_digest,
                            &LiveFrameProps {
                                viewport_height: super::super::recorded_paint::default_height(),
                                fit: FrameFit::default(),
                            },
                        )
                        .unwrap(),
                    ),
                }],
                event_ids: Arc::from([]),
            }));
            DecodedMessage::Snapshot(Snapshot::new(
                1,
                epoch,
                0,
                1,
                vec![Node::new(1, 0, 0, crate::KIND_VIEW), frame],
            ))
        };
        let command = |epoch, request_id, function_id, args: serde_json::Value| {
            DecodedMessage::Command(Command::new(
                CommandMeta {
                    surface_id: 1,
                    epoch,
                    after_revision: 1,
                    request_id,
                    node_id: 2,
                },
                CommandOperation::InvokeNative {
                    module_id: id,
                    module_digest: digest,
                    function_id,
                    args: crate::native::encode_native_request(build_digest, &args).unwrap(),
                },
            ))
        };
        let apply = |message, cx: &mut TestAppContext| {
            cx.update_window(window.into(), |_, window, cx| {
                root.update(cx, |root, cx| {
                    root.apply_decoded_message_in_window(message, window, cx)
                })
            })
            .unwrap()
            .unwrap();
        };
        apply(snapshot(1), cx);
        // Generated command IDs are alphabetical: replaceFrame is seventh.
        apply(
            command(
                1,
                1,
                7,
                serde_json::json!({"sequence":1,"width":1,"height":1,"rgba":[255,0,0,255]}),
            ),
            cx,
        );
        while let Some(event) = runtime.take_event().unwrap() {
            if let EventPayload::CommandResult(result) = event.payload {
                assert!(result.success);
            }
        }
        cx.update(|cx| assert_eq!(cx.global::<FrameBudget>().0.load(Ordering::Relaxed), 4));
        apply(snapshot(2), cx);
        cx.update(|cx| assert_eq!(cx.global::<FrameBudget>().0.load(Ordering::Relaxed), 0));
        apply(
            command(
                1,
                2,
                7,
                serde_json::json!({"sequence":2,"width":1,"height":1,"rgba":[0,255,0,255]}),
            ),
            cx,
        );
        let mut rejected = false;
        while let Some(event) = runtime.take_event().unwrap() {
            if let EventPayload::CommandResult(result) = event.payload {
                assert!(!result.success);
                rejected = true;
            }
        }
        assert!(rejected);
        apply(
            command(
                2,
                3,
                7,
                serde_json::json!({"sequence":1,"width":1,"height":1,"rgba":[0,0,255,255]}),
            ),
            cx,
        );
        apply(command(2, 4, 4, serde_json::Value::Null), cx);
        cx.update(|cx| assert_eq!(cx.global::<FrameBudget>().0.load(Ordering::Relaxed), 0));
    }

    #[gpui::test]
    fn paint_media_chunk_upload_clear_dispose_and_release_stay_within_host_budget(
        cx: &mut TestAppContext,
    ) {
        let old = cx.update(|cx| {
            let mut resource = FrameResource::default();
            resource
                .replace(
                    CpuFrame {
                        sequence: 1,
                        width: 1,
                        height: 1,
                        rgba: vec![255; 4],
                    },
                    cx,
                )
                .unwrap();
            let old = resource.current.as_ref().unwrap().downgrade();
            resource
                .begin(
                    FrameUpload {
                        sequence: 2,
                        width: 2,
                        height: 1,
                    },
                    cx,
                )
                .unwrap();
            assert!(resource.present(2, cx).is_err());
            assert!(
                resource
                    .write(FrameChunk {
                        sequence: 2,
                        offset: 1,
                        rgba: vec![1; 4]
                    })
                    .is_err()
            );
            resource
                .write(FrameChunk {
                    sequence: 2,
                    offset: 0,
                    rgba: vec![1; 4],
                })
                .unwrap();
            assert_eq!(
                resource.state().sequence,
                Some(1),
                "partial uploads never replace visible content"
            );
            resource
                .write(FrameChunk {
                    sequence: 2,
                    offset: 4,
                    rgba: vec![2; 4],
                })
                .unwrap();
            resource.present(2, cx).unwrap();
            assert_eq!(resource.state().retained_bytes, 8);
            resource.clear().unwrap();
            assert_eq!(resource.state().retained_bytes, 0);
            assert!(
                resource
                    .replace(
                        CpuFrame {
                            sequence: 2,
                            width: 1,
                            height: 1,
                            rgba: vec![0; 4]
                        },
                        cx
                    )
                    .is_err()
            );
            resource.dispose();
            assert!(
                resource
                    .begin(
                        FrameUpload {
                            sequence: 3,
                            width: 1,
                            height: 1
                        },
                        cx
                    )
                    .is_err()
            );
            old
        });
        assert!(old.upgrade().is_none());
        cx.update(|cx| assert_eq!(cx.global::<FrameBudget>().0.load(Ordering::Relaxed), 0));
        cx.update(|cx| {
            let mut resources: Vec<_> = (0..4).map(|_| FrameResource::default()).collect();
            for resource in &mut resources {
                resource
                    .begin(
                        FrameUpload {
                            sequence: 1,
                            width: 2048,
                            height: 2048,
                        },
                        cx,
                    )
                    .unwrap();
            }
            let mut excess = FrameResource::default();
            assert!(
                excess
                    .begin(
                        FrameUpload {
                            sequence: 1,
                            width: 1,
                            height: 1
                        },
                        cx
                    )
                    .is_err()
            );
            resources[0].cancel().unwrap();
            assert!(
                resources[0]
                    .begin(
                        FrameUpload {
                            sequence: 1,
                            width: 1,
                            height: 1
                        },
                        cx
                    )
                    .is_err()
            );
            excess
                .begin(
                    FrameUpload {
                        sequence: 1,
                        width: 1,
                        height: 1,
                    },
                    cx,
                )
                .unwrap();
        });
        cx.update(|cx| assert_eq!(cx.global::<FrameBudget>().0.load(Ordering::Relaxed), 0));
    }

    #[gpui::test]
    fn frame_replacement_is_atomic_bounded_and_monotonic(cx: &mut TestAppContext) {
        let mut resource = FrameResource::default();
        cx.update(|cx| {
            let first = CpuFrame {
                sequence: 1,
                width: 2,
                height: 1,
                rgba: vec![255, 0, 0, 255, 0, 255, 0, 255],
            };
            let state = resource.replace(first.clone(), cx).unwrap();
            assert_eq!(state.retained_bytes, 8);
            assert_eq!(state.sequence, Some(1));
            assert!(resource.replace(first, cx).is_err());
            assert!(
                resource
                    .replace(
                        CpuFrame {
                            sequence: 2,
                            width: 2,
                            height: 1,
                            rgba: vec![0; 4]
                        },
                        cx
                    )
                    .is_err()
            );
            assert_eq!(resource.state(), state);
            let latest = resource
                .replace(
                    CpuFrame {
                        sequence: 3,
                        width: 1,
                        height: 1,
                        rgba: vec![10, 20, 30, 255],
                    },
                    cx,
                )
                .unwrap();
            assert_eq!(latest.retained_bytes, 4);
            assert_eq!(
                resource
                    .current
                    .as_ref()
                    .unwrap()
                    .read(cx)
                    .image
                    .as_bytes(0)
                    .unwrap(),
                [30, 20, 10, 255]
            );
        });
    }
}
