use gpui::{
    AnyElement, App, Element, Entity, ImageCacheError, ImageRequest, ImageSource,
    InteractiveElement, ManagedImageFrame, ManagedImageSource, ObjectFit, Pixels, Size, Styled,
    StyledImage, Window, img,
};
use std::sync::Arc;

#[path = "image_decode.rs"]
mod decode;
#[path = "image_resource.rs"]
mod resource;
use resource::{ImageData, ImageKey, ImageVariant};

use crate::protocol::{HostProperties, Style};
use crate::tree::StoredNode;

use super::super::SolidRoot;

use super::accessibility::apply_accessibility;
use super::style::apply_style;

#[derive(Clone, PartialEq, Eq)]
struct ImageProvider {
    source: ImageKey,
    variants: Vec<(ImageKey, (u32, u32))>,
}

#[derive(Default)]
struct ImageState {
    identity: Option<ImageProvider>,
    source: Option<Entity<ImageData>>,
    variant: Option<Entity<ImageVariant>>,
    previous: Option<ManagedImageFrame>,
    intrinsic: Option<(u32, u32)>,
}

fn target_size(size: (u32, u32), request: ImageRequest, bucketed: bool) -> (u32, u32) {
    let width = f32::from(request.bounds.size.width).max(1.0);
    let height = f32::from(request.bounds.size.height).max(1.0);
    let x = width / size.0 as f32;
    let y = height / size.1 as f32;
    let (x, y) = match request.object_fit {
        ObjectFit::Fill => (x, y),
        ObjectFit::Contain => (x.min(y), x.min(y)),
        ObjectFit::Cover => (x.max(y), x.max(y)),
        ObjectFit::ScaleDown => (x.min(y).min(1.0), x.min(y).min(1.0)),
        ObjectFit::None => (1.0, 1.0),
    };
    // Quarter-octave buckets bound resize churn without doubling both axes.
    let bucket = |value: f32| {
        if !bucketed {
            return value.max(1.0).ceil().min(32768.0) as u32;
        }
        let exponent = (value.max(1.0).log2() * 4.0).ceil() / 4.0;
        2.0_f32.powf(exponent).ceil().min(32768.0) as u32
    };
    (
        bucket(size.0 as f32 * x * request.scale_factor),
        bucket(size.1 as f32 * y * request.scale_factor),
    )
}

impl ImageProvider {
    fn with_state<R>(
        &self,
        window: &mut Window,
        cx: &mut App,
        f: impl FnOnce(&mut ImageState, &mut Window, &mut App) -> R,
    ) -> R {
        window.with_global_id("solid-gpui-image-resource".into(), |id, window| {
            window.with_element_state(id, |state: Option<ImageState>, window| {
                let mut state = state.unwrap_or_default();
                if state.identity.as_ref() != Some(self) {
                    state = ImageState {
                        identity: Some(self.clone()),
                        ..Default::default()
                    };
                }
                let result = f(&mut state, window, cx);
                (result, state)
            })
        })
    }
}

impl ManagedImageSource for ImageProvider {
    fn intrinsic_size(
        &self,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Result<Size<Pixels>, ImageCacheError>> {
        self.with_state(window, cx, |state, window, cx| {
            // Explicit candidates provide metadata without downloading the original.
            if let Some((_, size)) = self
                .variants
                .iter()
                .max_by_key(|(_, size)| u64::from(size.0) * u64::from(size.1))
            {
                return Some(Ok(gpui::size(
                    gpui::px(size.0 as f32),
                    gpui::px(size.1 as f32),
                )));
            }
            if let Some(size) = state.intrinsic {
                return Some(Ok(gpui::size(
                    gpui::px(size.0 as f32),
                    gpui::px(size.1 as f32),
                )));
            }
            if state
                .source
                .as_ref()
                .is_none_or(|source| source.read(cx).key != self.source)
            {
                state.variant = None;
                state.source = Some(ImageData::acquire(&self.source, cx));
            }
            let result = state
                .source
                .as_ref()
                .unwrap()
                .update(cx, |source, _| source.get(window.current_view()));
            if let Some(Ok(encoded)) = &result {
                state.intrinsic = Some(encoded.size);
            }
            result.map(|result| {
                result.map(|encoded| {
                    gpui::size(
                        gpui::px(encoded.size.0 as f32),
                        gpui::px(encoded.size.1 as f32),
                    )
                })
            })
        })
    }

    fn image(
        &self,
        request: ImageRequest,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Result<ManagedImageFrame, ImageCacheError>> {
        self.with_state(window, cx, |state, window, cx| {
            if !request.visible {
                state.variant = None;
                state.previous = None;
                if state.intrinsic.is_some() || !self.variants.is_empty() {
                    state.source = None;
                }
                return None;
            }
            if !self.variants.is_empty() {
                let intrinsic = self
                    .variants
                    .iter()
                    .max_by_key(|(_, size)| u64::from(size.0) * u64::from(size.1))
                    .unwrap()
                    .1;
                let target = target_size(intrinsic, request, false);
                let selected = self
                    .variants
                    .iter()
                    .filter(|(_, size)| size.0 >= target.0 && size.1 >= target.1)
                    .min_by_key(|(_, size)| u64::from(size.0) * u64::from(size.1))
                    .or_else(|| {
                        self.variants
                            .iter()
                            .max_by_key(|(_, size)| u64::from(size.0) * u64::from(size.1))
                    })
                    .unwrap();
                if state
                    .source
                    .as_ref()
                    .is_none_or(|source| source.read(cx).key != selected.0)
                {
                    state.previous = state
                        .variant
                        .as_ref()
                        .and_then(|variant| variant.read(cx).current.clone())
                        .or_else(|| state.previous.take());
                    state.variant = None;
                    state.source = Some(ImageData::acquire(&selected.0, cx));
                }
            }
            if state.source.is_none() {
                state.source = Some(ImageData::acquire(&self.source, cx));
            }
            let source = state.source.as_ref()?;
            let encoded = match source.update(cx, |source, _| source.get(window.current_view())) {
                Some(Ok(encoded)) => encoded,
                Some(Err(error)) => {
                    state.previous = None;
                    return Some(Err(error));
                }
                None => return state.previous.clone().map(Ok),
            };
            let mut target = target_size(encoded.size, request, true);
            if !encoded.scalable {
                target = (target.0.min(encoded.size.0), target.1.min(encoded.size.1));
            }
            if state
                .variant
                .as_ref()
                .is_none_or(|variant| variant.read(cx).target != target)
            {
                state.previous = state
                    .variant
                    .as_ref()
                    .and_then(|variant| variant.read(cx).current.clone())
                    .or_else(|| state.previous.take());
                state.variant = Some(ImageVariant::acquire(source.clone(), encoded, target, cx));
            }
            let frame = state.variant.as_ref().unwrap().update(cx, |variant, cx| {
                variant.frame(window.current_view(), request.animate, cx)
            });
            if frame.is_some() {
                state.previous = None;
            }
            frame.or_else(|| state.previous.clone().map(Ok))
        })
    }
}

fn image_source(source: &str) -> ImageSource {
    ImageSource::Managed(Arc::new(ImageProvider {
        source: ImageKey::parse(source),
        variants: Vec::new(),
    }))
}

fn object_fit_from_code(code: u32) -> ObjectFit {
    match code {
        1 => ObjectFit::Fill,
        2 => ObjectFit::Contain,
        3 => ObjectFit::Cover,
        4 => ObjectFit::ScaleDown,
        5 => ObjectFit::None,
        _ => unreachable!("validated image objectFit"),
    }
}

pub(super) fn render(
    node: &StoredNode,
    entity: &Entity<SolidRoot>,
    style: Option<&Style>,
) -> AnyElement {
    let Some(HostProperties::Image(image)) = node.host_properties.as_ref() else {
        return gpui::div()
            .id(gpui::ElementId::Integer(node.id as u64))
            .into_any();
    };
    let object_fit_code = image.object_fit;
    let image_id = gpui::ElementId::named_usize("solid-gpui-image", node.id as usize);
    let source = ImageSource::Managed(Arc::new(ImageProvider {
        source: ImageKey::parse(&image.source),
        variants: image
            .sources
            .iter()
            .map(|candidate| {
                (
                    ImageKey::parse(&candidate.source),
                    (candidate.width, candidate.height),
                )
            })
            .collect(),
    }));
    let mut image_element = img(source)
        .id(image_id)
        .object_fit(object_fit_from_code(object_fit_code));
    if let Some(fallback_source) = image.fallback_source.as_ref() {
        let fallback = image_source(fallback_source);
        let radius = gpui::px(style.and_then(|style| style.border_radius).unwrap_or(0.0));
        let fallback_id =
            gpui::ElementId::named_usize("solid-gpui-image-fallback", node.id as usize);
        image_element = image_element.with_fallback(move || {
            img(fallback.clone())
                .size_full()
                .rounded(radius)
                .id(fallback_id.clone())
                .object_fit(object_fit_from_code(object_fit_code))
                .into_any()
        });
    }
    let image_element = apply_style(image_element, style);
    super::measure_node(
        node,
        apply_accessibility(image_element, node).into_any(),
        entity,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt as _;
    use gpui::{AppContext, Asset, ImageAssetLoader, RenderImage, Resource};
    use resource::Images;
    use std::io::Cursor;

    const INLINE_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAABCAYAAAD0In+KAAAADklEQVR4nGP4z8DwHwQBEPgD/U6VwW8AAAAASUVORK5CYII=";

    #[gpui::test]
    fn image_fetches_are_bounded_and_queued_loads_cancel(cx: &mut gpui::TestAppContext) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let starts = Arc::new(AtomicUsize::new(0));
        let count = starts.clone();
        cx.update(|cx| {
            cx.set_http_client(gpui::http_client::FakeHttpClient::create(move |_| {
                count.fetch_add(1, Ordering::SeqCst);
                async {
                    std::future::pending::<()>().await;
                    Ok(gpui::http_client::Response::builder()
                        .status(200)
                        .body(Vec::new().into())
                        .unwrap())
                }
            }))
        });
        let leases = cx.update(|cx| {
            (0..12)
                .map(|i| {
                    ImageData::acquire(
                        &ImageKey::parse(&format!("https://images.example/{i}.png")),
                        cx,
                    )
                })
                .collect::<Vec<_>>()
        });
        cx.run_until_parked();
        assert_eq!(starts.load(Ordering::SeqCst), 4);
        drop(leases);
        cx.update(|_| {});
        cx.run_until_parked();
        let settled = starts.load(Ordering::SeqCst);
        cx.run_until_parked();
        assert_eq!(
            starts.load(Ordering::SeqCst),
            settled,
            "released owners must leave no runnable queued fetches"
        );
        cx.update(|cx| assert!(cx.global::<Images>().sources.is_empty()));
    }

    fn loaded_pixels(source: &str, cx: &App) -> Arc<RenderImage> {
        let source = cx.global::<Images>().sources[&ImageKey::parse(source)]
            .upgrade()
            .unwrap();
        let variant = cx
            .global::<Images>()
            .variants
            .iter()
            .find(|(key, _)| key.0 == source.entity_id())
            .unwrap()
            .1
            .upgrade()
            .unwrap();
        variant.read(cx).current.as_ref().unwrap().image.clone()
    }

    #[test]
    fn decode_target_respects_fit_dpi_and_resize_buckets() {
        let mut request = ImageRequest {
            bounds: gpui::Bounds::new(
                gpui::point(gpui::px(0.), gpui::px(0.)),
                gpui::size(gpui::px(200.), gpui::px(150.)),
            ),
            object_fit: ObjectFit::Contain,
            scale_factor: 2.,
            animate: false,
            visible: true,
        };
        let contain = target_size((4000, 3000), request, true);
        assert!(contain.0 >= 400 && contain.0 < 480 && contain.1 >= 300 && contain.1 < 360);
        request.bounds.size.width = gpui::px(201.);
        assert_eq!(target_size((4000, 3000), request, true), contain);
        request.bounds.size.height = gpui::px(200.);
        request.object_fit = ObjectFit::Cover;
        let cover = target_size((4000, 3000), request, true);
        assert!(
            cover.0 >= 534 && cover.1 >= 400,
            "cover must retain pixels beyond the crop"
        );
        request.scale_factor = 1.;
        let low_dpi = target_size((4000, 3000), request, true);
        assert!(low_dpi.0 < cover.0 && low_dpi.1 < cover.1);
    }

    #[gpui::test]
    fn natural_size_loads_and_failed_candidate_recovers_after_resize(
        cx: &mut gpui::TestAppContext,
    ) {
        let window = cx.open_window(gpui::size(gpui::px(400.), gpui::px(400.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        let commit = |cx: &mut gpui::TestAppContext,
                      source: &str,
                      style: Option<Style>,
                      sources: Vec<crate::ImageCandidate>| {
            window
                .update(cx, |root, window, cx| {
                    let mut node = crate::Node::new(2, 1, 0, crate::tree::KIND_IMAGE);
                    node.style = style;
                    node.host_properties = Some(HostProperties::Image(crate::ImageProperties {
                        source: source.into(),
                        object_fit: 2,
                        fallback_source: Some(INLINE_PNG.into()),
                        sources,
                    }));
                    let revision = root.store().revision();
                    root.apply_decoded_message_in_window(
                        crate::protocol::DecodedMessage::Snapshot(crate::Snapshot::new(
                            1,
                            1,
                            revision,
                            revision + 1,
                            vec![crate::Node::new(1, 0, 0, crate::tree::KIND_VIEW), node],
                        )),
                        window,
                        cx,
                    )
                    .unwrap();
                })
                .unwrap();
            draw(cx, window);
        };
        commit(cx, INLINE_PNG, None, Vec::new());
        window
            .update(cx, |_, window, cx| {
                assert!(window.has_image_atlas_entry(&loaded_pixels(INLINE_PNG, cx)))
            })
            .unwrap();
        let candidates = vec![
            crate::ImageCandidate {
                source: INLINE_PNG.into(),
                width: 2,
                height: 1,
            },
            crate::ImageCandidate {
                source: "/missing-large-variant.png".into(),
                width: 400,
                height: 200,
            },
        ];
        commit(
            cx,
            INLINE_PNG,
            Some(Style {
                width: Some(100.),
                height: Some(50.),
                ..Default::default()
            }),
            candidates.clone(),
        );
        commit(
            cx,
            INLINE_PNG,
            Some(Style {
                width: Some(1.),
                height: Some(0.5),
                ..Default::default()
            }),
            candidates,
        );
        window
            .update(cx, |_, window, cx| {
                assert!(window.has_image_atlas_entry(&loaded_pixels(INLINE_PNG, cx)))
            })
            .unwrap();
        cx.update(|cx| {
            assert!(
                !cx.global::<Images>()
                    .sources
                    .contains_key(&ImageKey::parse("/missing-large-variant.png"))
            )
        });
    }

    #[gpui::test]
    fn source_set_loads_only_the_smallest_sufficient_variant(cx: &mut gpui::TestAppContext) {
        let window = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        let svg = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='200'%3E%3Crect width='200' height='200' fill='red'/%3E%3C/svg%3E";
        window
            .update(cx, |root, window, cx| {
                let mut node = crate::Node::new(2, 1, 0, crate::tree::KIND_IMAGE);
                node.style = Some(Style {
                    width: Some(100.),
                    height: Some(100.),
                    ..Default::default()
                });
                node.host_properties = Some(HostProperties::Image(crate::ImageProperties {
                    source: "/must-not-load-original.png".into(),
                    object_fit: 2,
                    fallback_source: None,
                    sources: vec![
                        crate::ImageCandidate {
                            source: svg.into(),
                            width: 200,
                            height: 200,
                        },
                        crate::ImageCandidate {
                            source: "/must-not-load-large.png".into(),
                            width: 4000,
                            height: 4000,
                        },
                    ],
                }));
                root.apply_decoded_message_in_window(
                    crate::protocol::DecodedMessage::Snapshot(crate::Snapshot::new(
                        1,
                        1,
                        0,
                        1,
                        vec![crate::Node::new(1, 0, 0, crate::tree::KIND_VIEW), node],
                    )),
                    window,
                    cx,
                )
                .unwrap();
            })
            .unwrap();
        draw(cx, window);
        window
            .update(cx, |_, window, cx| {
                let pixels = loaded_pixels(svg, cx);
                assert!(
                    pixels.size(0).width.0 >= 200
                        && pixels.size(0).width.0 < 240
                        && pixels.size(0).height.0 >= 200
                        && pixels.size(0).height.0 < 240
                );
                assert!(window.has_image_atlas_entry(&pixels));
                assert!(
                    !cx.global::<Images>()
                        .sources
                        .contains_key(&ImageKey::parse("/must-not-load-original.png"))
                );
                assert!(
                    !cx.global::<Images>()
                        .sources
                        .contains_key(&ImageKey::parse("/must-not-load-large.png"))
                );
            })
            .unwrap();
    }

    #[gpui::test]
    fn animated_images_advance_without_retaining_old_frames(cx: &mut gpui::TestAppContext) {
        let mut bytes = Vec::new();
        {
            let mut encoder = gif::Encoder::new(&mut bytes, 2, 1, &[255, 0, 0, 0, 255, 0]).unwrap();
            for index in 0..40 {
                let frame = gif::Frame {
                    width: 2,
                    height: 1,
                    delay: 5,
                    buffer: std::borrow::Cow::Owned(vec![(index % 2) as u8; 2]),
                    ..Default::default()
                };
                encoder.write_frame(&frame).unwrap();
            }
        }
        let source = format!(
            "data:image/gif,{}",
            bytes
                .iter()
                .map(|byte| format!("%{byte:02X}"))
                .collect::<String>()
        );
        let window = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        publish(cx, window, Some(&source), None);
        let first = cx.update(|cx| loaded_pixels(&source, cx));
        assert_eq!(
            first.as_bytes(0).unwrap(),
            &[0, 0, 255, 255, 0, 0, 255, 255]
        );
        let weak = Arc::downgrade(&first);
        drop(first);
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(60));
        draw(cx, window);
        cx.update(|cx| {
            let next = loaded_pixels(&source, cx);
            assert_eq!(next.as_bytes(0).unwrap(), &[0, 255, 0, 255, 0, 255, 0, 255]);
            assert_eq!(
                next.frame_count(),
                1,
                "the renderer receives one animation frame, not the whole sequence"
            );
        });
        assert!(
            weak.upgrade().is_none(),
            "presented old frame is released after all windows advance"
        );
        publish(cx, window, None, None);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(10));
        cx.run_until_parked();
        cx.update(|cx| assert!(cx.global::<Images>().sources.is_empty()));
    }

    fn draw(cx: &mut gpui::TestAppContext, window: gpui::WindowHandle<SolidRoot>) {
        for _ in 0..3 {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
    }

    fn publish(
        cx: &mut gpui::TestAppContext,
        window: gpui::WindowHandle<SolidRoot>,
        source: Option<&str>,
        fallback: Option<&str>,
    ) {
        window
            .update(cx, |root, window, cx| {
                let mut nodes = vec![crate::Node::new(1, 0, 0, crate::tree::KIND_VIEW)];
                if let Some(source) = source {
                    let mut node = crate::Node::new(2, 1, 0, crate::tree::KIND_IMAGE);
                    node.style = Some(Style {
                        width: Some(100.),
                        height: Some(100.),
                        ..Default::default()
                    });
                    node.host_properties = Some(HostProperties::Image(crate::ImageProperties {
                        source: source.into(),
                        object_fit: 2,
                        fallback_source: fallback.map(Into::into),
                        sources: Vec::new(),
                    }));
                    nodes.push(node);
                }
                let revision = root.store().revision();
                let snapshot = crate::Snapshot::new(1, 1, revision, revision + 1, nodes);
                root.apply_decoded_message_in_window(
                    crate::protocol::DecodedMessage::Snapshot(snapshot),
                    window,
                    cx,
                )
                .unwrap();
            })
            .unwrap();
        draw(cx, window);
    }

    #[gpui::test]
    fn images_share_across_windows_and_release_pixels_and_atlases(cx: &mut gpui::TestAppContext) {
        let first = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        let second = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        publish(cx, first, Some(INLINE_PNG), None);
        let (lease, pixels) = cx.update(|cx| {
            let lease = cx.global::<Images>().sources[&ImageKey::parse(INLINE_PNG)].clone();
            let image = loaded_pixels(INLINE_PNG, cx);
            (lease, image)
        });
        publish(cx, second, Some(INLINE_PNG), None);
        for window in [first, second] {
            window
                .update(cx, |_, window, _| {
                    assert!(window.has_image_atlas_entry(&pixels))
                })
                .unwrap();
        }
        publish(cx, first, None, None);
        assert!(
            lease.upgrade().is_some(),
            "another window still owns the image"
        );
        second
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        cx.run_until_parked();
        draw(cx, first);
        assert!(
            lease.upgrade().is_none(),
            "closing the last image window releases the load"
        );
        first
            .update(cx, |_, window, cx| {
                assert!(
                    !window.has_image_atlas_entry(&pixels),
                    "all windows must release old atlas tiles"
                );
                assert!(cx.global::<Images>().sources.is_empty());
            })
            .unwrap();
        let weak_pixels = Arc::downgrade(&pixels);
        drop(pixels);
        assert!(
            weak_pixels.upgrade().is_none(),
            "decoded pixels must not enter the global asset cache"
        );
    }

    #[gpui::test]
    fn changed_primary_does_not_load_new_fallback_until_its_decode_fails(
        cx: &mut gpui::TestAppContext,
    ) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        const FAILED: &str = "https://images.example/failed.png";
        const PENDING: &str = "https://images.example/pending.png";
        const FALLBACK: &str = "https://images.example/fallback.png";
        let (release, pending) = futures::channel::oneshot::channel::<()>();
        let pending = futures::lock::Mutex::new(Some(pending));
        let fallback_fetches = Arc::new(AtomicUsize::new(0));
        let fetches = fallback_fetches.clone();
        let fallback_bytes = data_url::DataUrl::process(INLINE_PNG)
            .unwrap()
            .decode_to_vec()
            .unwrap()
            .0;
        cx.update(|cx| {
            cx.set_http_client(gpui::http_client::FakeHttpClient::create(move |request| {
                let uri = request.uri().to_owned();
                let pending = if uri == PENDING {
                    Some(
                        pending
                            .try_lock()
                            .expect("uncontended receiver")
                            .take()
                            .unwrap(),
                    )
                } else {
                    None
                };
                if uri == FALLBACK {
                    fetches.fetch_add(1, Ordering::SeqCst);
                }
                let bytes = fallback_bytes.clone();
                async move {
                    if let Some(pending) = pending {
                        let _ = pending.await;
                    }
                    let response = gpui::http_client::Response::builder();
                    if uri == FAILED {
                        Ok(response.status(404).body(Vec::new().into()).unwrap())
                    } else if uri == PENDING {
                        // The HTTP request succeeds, but the selected primary fails decoding.
                        Ok(response
                            .status(200)
                            .body(b"invalid png".to_vec().into())
                            .unwrap())
                    } else {
                        assert_eq!(uri, FALLBACK);
                        Ok(response.status(200).body(bytes.into()).unwrap())
                    }
                }
            }))
        });
        let window = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        publish(cx, window, Some(FAILED), Some(INLINE_PNG));
        assert_eq!(fallback_fetches.load(Ordering::SeqCst), 0);

        publish(cx, window, Some(PENDING), Some(FALLBACK));
        assert_eq!(fallback_fetches.load(Ordering::SeqCst), 0);
        cx.update(|cx| {
            assert!(
                !cx.global::<Images>()
                    .sources
                    .contains_key(&ImageKey::parse(FALLBACK))
            );
        });

        release.send(()).unwrap();
        draw(cx, window);
        assert_eq!(fallback_fetches.load(Ordering::SeqCst), 1);
        window
            .update(cx, |_, window, cx| {
                let fallback = loaded_pixels(FALLBACK, cx);
                assert!(window.has_image_atlas_entry(&fallback));
            })
            .unwrap();
    }

    #[gpui::test]
    fn resized_source_set_does_not_load_fallback_while_new_candidate_is_pending(
        cx: &mut gpui::TestAppContext,
    ) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        const SMALL: &str = "https://images.example/small.png";
        const LARGE: &str = "https://images.example/large.png";
        const FALLBACK: &str = "https://images.example/resize-fallback.png";
        let (release, pending) = futures::channel::oneshot::channel::<()>();
        let pending = futures::lock::Mutex::new(Some(pending));
        let fallback_fetches = Arc::new(AtomicUsize::new(0));
        let fetches = fallback_fetches.clone();
        cx.update(|cx| {
            cx.set_http_client(gpui::http_client::FakeHttpClient::create(move |request| {
                let uri = request.uri().to_owned();
                let pending = if uri == LARGE {
                    Some(
                        pending
                            .try_lock()
                            .expect("uncontended receiver")
                            .take()
                            .unwrap(),
                    )
                } else {
                    None
                };
                if uri == FALLBACK {
                    fetches.fetch_add(1, Ordering::SeqCst);
                }
                async move {
                    if let Some(pending) = pending {
                        let _ = pending.await;
                    }
                    Ok(gpui::http_client::Response::builder()
                        .status(404)
                        .body(Vec::new().into())
                        .unwrap())
                }
            }))
        });
        let window = cx.open_window(gpui::size(gpui::px(400.), gpui::px(400.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        let commit = |cx: &mut gpui::TestAppContext, width: f32, fallback: &str| {
            window
                .update(cx, |root, window, cx| {
                    let mut node = crate::Node::new(2, 1, 0, crate::tree::KIND_IMAGE);
                    node.style = Some(Style {
                        width: Some(width),
                        height: Some(width),
                        ..Default::default()
                    });
                    node.host_properties = Some(HostProperties::Image(crate::ImageProperties {
                        source: SMALL.into(),
                        object_fit: 2,
                        fallback_source: Some(fallback.into()),
                        sources: vec![
                            crate::ImageCandidate {
                                source: SMALL.into(),
                                width: 100,
                                height: 100,
                            },
                            crate::ImageCandidate {
                                source: LARGE.into(),
                                width: 400,
                                height: 400,
                            },
                        ],
                    }));
                    let revision = root.store().revision();
                    root.apply_decoded_message_in_window(
                        crate::protocol::DecodedMessage::Snapshot(crate::Snapshot::new(
                            1,
                            1,
                            revision,
                            revision + 1,
                            vec![crate::Node::new(1, 0, 0, crate::tree::KIND_VIEW), node],
                        )),
                        window,
                        cx,
                    )
                    .unwrap();
                })
                .unwrap();
            draw(cx, window);
        };
        commit(cx, 40., INLINE_PNG);
        commit(cx, 300., FALLBACK);
        assert_eq!(fallback_fetches.load(Ordering::SeqCst), 0);
        cx.update(|cx| {
            assert!(
                !cx.global::<Images>()
                    .sources
                    .contains_key(&ImageKey::parse(FALLBACK))
            );
        });
        release.send(()).unwrap();
        draw(cx, window);
        assert_eq!(fallback_fetches.load(Ordering::SeqCst), 1);
    }

    #[gpui::test]
    fn replacing_primary_releases_fallback_without_eager_loading(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| cx.set_http_client(gpui::http_client::FakeHttpClient::with_404_response()));
        let window = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        publish(
            cx,
            window,
            Some("https://images.example/missing.png"),
            Some(INLINE_PNG),
        );
        let old = cx.update(|cx| Arc::downgrade(&loaded_pixels(INLINE_PNG, cx)));
        let replacement = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='2' height='2'%3E%3Crect width='2' height='2' fill='red'/%3E%3C/svg%3E";
        publish(cx, window, Some(replacement), Some(INLINE_PNG));
        assert!(
            old.upgrade().is_none(),
            "an unused fallback must release its decoded image"
        );
        publish(cx, window, None, None);
        cx.update(|cx| assert!(cx.global::<Images>().sources.is_empty()));
    }

    #[gpui::test]
    fn removing_last_image_cancels_pending_http_load(cx: &mut gpui::TestAppContext) {
        let (sender, receiver) = futures::channel::oneshot::channel::<()>();
        let receiver = futures::lock::Mutex::new(Some(receiver));
        cx.update(|cx| {
            cx.set_http_client(gpui::http_client::FakeHttpClient::create(move |_| {
                let receiver = receiver
                    .try_lock()
                    .expect("uncontended test receiver")
                    .take()
                    .expect("one shared load");
                async move {
                    let _ = receiver.await;
                    Ok(gpui::http_client::Response::builder()
                        .status(404)
                        .body(Vec::new().into())
                        .unwrap())
                }
            }))
        });
        let window = cx.open_window(gpui::size(gpui::px(200.), gpui::px(200.)), |_, _| {
            SolidRoot::new(crate::InMemoryAdapter::new())
        });
        publish(cx, window, Some("https://images.example/pending.png"), None);
        assert!(!sender.is_canceled());
        publish(cx, window, None, None);
        assert!(
            sender.is_canceled(),
            "a detached image must not leave a download running"
        );
    }

    #[gpui::test]
    async fn remote_image_source_reaches_http_loader_and_decodes_pixels(
        cx: &mut gpui::TestAppContext,
    ) {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255])
            .unwrap()
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let encoded = encoded.into_inner();
        cx.update(|cx| {
            cx.set_http_client(gpui::http_client::FakeHttpClient::create(move |request| {
                assert_eq!(request.uri(), "https://images.example/fixture.png?size=2");
                let encoded = encoded.clone();
                async move {
                    Ok(gpui::http_client::Response::builder()
                        .status(200)
                        .body(encoded.into())
                        .unwrap())
                }
            }))
        });
        let ImageKey::Resource(resource) =
            ImageKey::parse("https://images.example/fixture.png?size=2")
        else {
            panic!("remote images must use the resource cache");
        };
        let image = cx
            .update(|cx| {
                let future: futures::future::BoxFuture<'static, _> =
                    ImageAssetLoader::load(resource, cx).boxed();
                cx.background_spawn(future)
            })
            .await
            .expect("remote image should load");
        assert_eq!(
            image.size(0),
            gpui::size(gpui::DevicePixels(2), gpui::DevicePixels(1))
        );
        assert_eq!(
            image.as_bytes(0).unwrap(),
            &[0, 0, 255, 255, 0, 255, 0, 255]
        );
        assert!(matches!(
            ImageKey::parse("assets/photo.png"),
            ImageKey::Resource(Resource::Path(_))
        ));
    }
    #[gpui::test]
    async fn file_urls_and_inline_fallback_reach_the_native_image_atlas(
        cx: &mut gpui::TestAppContext,
    ) {
        let (png, _) = data_url::DataUrl::process(INLINE_PNG)
            .unwrap()
            .decode_to_vec()
            .unwrap();
        let directory =
            std::env::temp_dir().join(format!("solid-gpui-image-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("image #雪.png");
        std::fs::write(&path, png).unwrap();
        let file_url = gpui::http_client::Url::from_file_path(&path).unwrap();
        let ImageKey::Resource(Resource::Path(resolved)) = ImageKey::parse(file_url.as_str())
        else {
            panic!("file URL must resolve to a decoded native path");
        };
        assert_eq!(resolved.as_ref(), path);
        let loaded = cx
            .update(|cx| {
                let future: futures::future::BoxFuture<'static, _> =
                    ImageAssetLoader::load(Resource::Path(resolved), cx).boxed();
                cx.background_spawn(future)
            })
            .await
            .unwrap();
        assert_eq!(
            loaded.as_bytes(0).unwrap(),
            &[0, 0, 255, 255, 0, 255, 0, 255]
        );
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();

        cx.update(|cx| cx.set_http_client(gpui::http_client::FakeHttpClient::with_404_response()));
        let runtime = crate::InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(gpui::px(240.), gpui::px(160.)), move |_, _| {
            SolidRoot::new(runtime)
        });
        let root = window.root(cx).unwrap();
        let mut node = crate::Node::new(2, 1, 0, crate::tree::KIND_IMAGE);
        node.style = Some(Style {
            width: Some(120.),
            height: Some(80.),
            ..Default::default()
        });
        node.host_properties = Some(HostProperties::Image(crate::ImageProperties {
            source: "https://images.example/missing.png".into(),
            object_fit: 2,
            fallback_source: Some(INLINE_PNG.into()),
            sources: Vec::new(),
        }));
        let snapshot = crate::Snapshot::new(
            1,
            1,
            0,
            1,
            vec![crate::Node::new(1, 0, 0, crate::tree::KIND_VIEW), node],
        );
        root.update(cx, |root, cx| {
            root.apply_payload(&snapshot.encode().unwrap(), cx)
        })
        .unwrap();
        for _ in 0..4 {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
        cx.update_window(window.into(), |_, window, cx| {
            let image = loaded_pixels(INLINE_PNG, cx);
            assert!(
                window.has_image_atlas_entry(&image),
                "fallback must reach the native paint path"
            );
            assert_eq!(image.as_bytes(0).unwrap(), loaded.as_bytes(0).unwrap());
        })
        .unwrap();
    }
}
