use std::path::PathBuf;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Asset, Element, Entity, Image, ImageCacheError, ImageFormat, ImageSource,
    InteractiveElement, ObjectFit, RenderImage, Resource, SharedString, Styled, StyledImage, img,
};

use crate::protocol::{HostProperties, Style};
use crate::tree::StoredNode;

use super::super::SolidRoot;

use super::accessibility::apply_accessibility;
use super::style::apply_style;

struct InlineImage;
impl Asset for InlineImage {
    type Source = SharedString;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;
    fn load(
        source: SharedString,
        cx: &mut App,
    ) -> impl Future<Output = Self::Output> + Send + 'static {
        let svg = cx.svg_renderer();
        async move {
            if source.len() > crate::protocol::MAX_IMAGE_SOURCE_BYTES {
                return Err(image_error("Image data URL exceeds the source byte limit"));
            }
            let data = data_url::DataUrl::process(&source)
                .map_err(|_| image_error("Invalid image data URL"))?;
            let mime = data.mime_type();
            let format = ImageFormat::from_mime_type(&format!("{}/{}", mime.type_, mime.subtype))
                .ok_or_else(|| image_error("Unsupported inline image media type"))?;
            let (bytes, _) = data
                .decode_to_vec()
                .map_err(|_| image_error("Invalid image data encoding"))?;
            Image::from_bytes(format, bytes)
                .to_image_data(svg)
                .map_err(Into::into)
        }
    }
}

fn image_error(message: &'static str) -> ImageCacheError {
    ImageCacheError::Asset(message.into())
}

fn image_source(source: &str) -> ImageSource {
    if source
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
    {
        let source = SharedString::from(source.to_owned());
        return ImageSource::Custom(Arc::new(move |window, cx| {
            window.use_asset::<InlineImage>(&source, cx)
        }));
    }
    if source.contains("://")
        || source
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("file:"))
    {
        let resource =
            gpui::http_client::Url::parse(source)
                .ok()
                .and_then(|url| match url.scheme() {
                    "http" | "https" => Some(Resource::Uri(url.to_string().into())),
                    #[cfg(not(target_family = "wasm"))]
                    "file" => url.to_file_path().ok().map(Into::into),
                    _ => None,
                });
        return match resource {
            Some(resource) => ImageSource::Resource(resource),
            None => ImageSource::Custom(Arc::new(|_, _| {
                Some(Err(image_error("Invalid or unsupported image URI")))
            })),
        };
    }
    ImageSource::from(PathBuf::from(source))
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
    let mut image_element = img(image_source(&image.source))
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
    use gpui::{AppContext as _, Asset, ImageAssetLoader, Resource};
    use std::io::Cursor;

    const INLINE_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAABCAYAAAD0In+KAAAADklEQVR4nGP4z8DwHwQBEPgD/U6VwW8AAAAASUVORK5CYII=";

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
        let ImageSource::Resource(resource) =
            image_source("https://images.example/fixture.png?size=2")
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
            image_source("assets/photo.png"),
            ImageSource::Resource(Resource::Path(_))
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
        let ImageSource::Resource(Resource::Path(resolved)) = image_source(file_url.as_str())
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
            let image = window
                .get_asset::<InlineImage>(&SharedString::from(INLINE_PNG), cx)
                .expect("inline fallback must be requested")
                .expect("inline fallback must decode");
            assert!(
                window.has_image_atlas_entry(&image),
                "fallback must reach the native paint path"
            );
            assert_eq!(image.as_bytes(0).unwrap(), loaded.as_bytes(0).unwrap());
        })
        .unwrap();
    }
}
