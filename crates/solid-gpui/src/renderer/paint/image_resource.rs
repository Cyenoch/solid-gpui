use super::decode::{DecodedFrame, EncodedImage, ImageDecoder};
use futures::AsyncReadExt;
use gpui::{
    App, AppContext, Context, Entity, EntityId, Global, ImageCacheError, ManagedImageFrame,
    RenderImage, Resource, SharedString, Task, WeakEntity,
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Semaphore;

const MAX_SOURCE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum ImageKey {
    Resource(Resource),
    Inline(SharedString),
    Invalid,
}
impl ImageKey {
    pub(super) fn parse(source: &str) -> Self {
        if source
            .get(..5)
            .is_some_and(|p| p.eq_ignore_ascii_case("data:"))
        {
            return Self::Inline(source.to_owned().into());
        }
        if source.contains("://")
            || source
                .get(..5)
                .is_some_and(|p| p.eq_ignore_ascii_case("file:"))
        {
            return gpui::http_client::Url::parse(source)
                .ok()
                .and_then(|url| match url.scheme() {
                    "http" | "https" => Some(Resource::Uri(url.to_string().into())),
                    #[cfg(not(target_family = "wasm"))]
                    "file" => url.to_file_path().ok().map(Into::into),
                    _ => None,
                })
                .map(Self::Resource)
                .unwrap_or(Self::Invalid);
        }
        Self::Resource(PathBuf::from(source).into())
    }
}

pub(super) struct Images {
    pub(super) sources: HashMap<ImageKey, WeakEntity<ImageData>>,
    pub(super) variants: HashMap<(EntityId, (u32, u32)), WeakEntity<ImageVariant>>,
    fetches: Arc<Semaphore>,
    decodes: Arc<Semaphore>,
}
impl Default for Images {
    fn default() -> Self {
        Self {
            sources: HashMap::new(),
            variants: HashMap::new(),
            fetches: Arc::new(Semaphore::new(4)),
            decodes: Arc::new(Semaphore::new(2)),
        }
    }
}
impl Global for Images {}

pub(super) struct ImageData {
    pub(super) key: ImageKey,
    pub(super) result: Option<Result<Arc<EncodedImage>, ImageCacheError>>,
    waiting: HashSet<EntityId>,
    _load: Task<()>,
}
impl ImageData {
    pub(super) fn acquire(key: &ImageKey, cx: &mut App) -> Entity<Self> {
        if !cx.has_global::<Images>() {
            cx.set_global(Images::default());
        }
        if let Some(existing) = cx
            .global::<Images>()
            .sources
            .get(key)
            .and_then(WeakEntity::upgrade)
        {
            return existing;
        }
        let client = cx.http_client();
        let svg = cx.svg_renderer();
        let assets = cx.asset_source().clone();
        let fetches = cx.global::<Images>().fetches.clone();
        let decodes = cx.global::<Images>().decodes.clone();
        let source = key.clone();
        let work = cx.background_executor().spawn(async move {
            let _permit = fetches.acquire().await.expect("image permits remain open");
            let bytes = match source {
                ImageKey::Resource(Resource::Path(path)) => {
                    let file = std::fs::File::open(path.as_ref())?;
                    let mut bytes = Vec::new();
                    file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut bytes)?;
                    bytes
                }
                ImageKey::Resource(Resource::Uri(uri)) => {
                    let mut response = client
                        .get(uri.as_ref(), ().into(), true)
                        .await
                        .map_err(ImageCacheError::from)?;
                    if !response.status().is_success() {
                        return Err(ImageCacheError::BadStatus {
                            uri,
                            status: response.status(),
                            body: String::new(),
                        });
                    }
                    let mut bytes = Vec::new();
                    response
                        .body_mut()
                        .take(MAX_SOURCE_BYTES + 1)
                        .read_to_end(&mut bytes)
                        .await?;
                    bytes
                }
                ImageKey::Resource(Resource::Embedded(path)) => assets
                    .load(&path)
                    .map_err(ImageCacheError::from)?
                    .ok_or_else(|| ImageCacheError::Asset("Embedded image is unavailable".into()))?
                    .into_owned(),
                ImageKey::Inline(source) => {
                    data_url::DataUrl::process(&source)
                        .map_err(|_| ImageCacheError::Asset("Invalid image data URL".into()))?
                        .decode_to_vec()
                        .map_err(|_| ImageCacheError::Asset("Invalid image data encoding".into()))?
                        .0
                }
                ImageKey::Invalid => {
                    return Err(ImageCacheError::Asset(
                        "Invalid or unsupported image URI".into(),
                    ));
                }
            };
            if bytes.len() as u64 > MAX_SOURCE_BYTES {
                return Err(ImageCacheError::Asset("Image source exceeds 32 MiB".into()));
            }
            drop(_permit);
            let _permit = decodes.acquire().await.expect("image permits remain open");
            EncodedImage::new(bytes, svg).map(Arc::new)
        });
        let entity = cx.new(|cx| {
            let id = cx.entity_id();
            cx.on_release(move |data: &mut Self, cx| {
                let sources = &mut cx.global_mut::<Images>().sources;
                if sources
                    .get(&data.key)
                    .is_some_and(|entry| entry.entity_id() == id)
                {
                    sources.remove(&data.key);
                }
            })
            .detach();
            Self {
                key: key.clone(),
                result: None,
                waiting: HashSet::new(),
                _load: cx.spawn(async move |this, cx| {
                    let result = work.await;
                    let _ = this.update(cx, |this: &mut Self, cx| {
                        this.result = Some(result);
                        for view in this.waiting.drain() {
                            App::notify(cx, view);
                        }
                    });
                }),
            }
        });
        cx.global_mut::<Images>()
            .sources
            .insert(key.clone(), entity.downgrade());
        entity
    }
    pub(super) fn get(
        &mut self,
        view: EntityId,
    ) -> Option<Result<Arc<EncodedImage>, ImageCacheError>> {
        if self.result.is_none() {
            self.waiting.insert(view);
        }
        self.result.clone()
    }
}

struct ImagePixels(Arc<RenderImage>);
impl ImagePixels {
    fn own(image: Arc<RenderImage>, cx: &mut App) -> ManagedImageFrame {
        let pixels = cx.new(|cx| {
            cx.on_release(|pixels: &mut Self, cx| cx.drop_image(pixels.0.clone(), None))
                .detach();
            Self(image.clone())
        });
        ManagedImageFrame {
            image,
            owner: pixels.into_any(),
        }
    }
}

pub(super) struct ImageVariant {
    _source: Entity<ImageData>,
    pub(super) target: (u32, u32),
    encoded: Arc<EncodedImage>,
    decoder: Option<ImageDecoder>,
    pub(super) current: Option<ManagedImageFrame>,
    next: Option<(ManagedImageFrame, Duration)>,
    delay: Duration,
    due: Option<web_time::Instant>,
    waiting: HashSet<EntityId>,
    animated: bool,
    pub(super) error: Option<ImageCacheError>,
    work: Option<Task<()>>,
    wake: Option<Task<()>>,
}
impl ImageVariant {
    pub(super) fn acquire(
        source: Entity<ImageData>,
        encoded: Arc<EncodedImage>,
        target: (u32, u32),
        cx: &mut App,
    ) -> Entity<Self> {
        let key = (source.entity_id(), target);
        if let Some(existing) = cx
            .global::<Images>()
            .variants
            .get(&key)
            .and_then(WeakEntity::upgrade)
        {
            return existing;
        }
        let entity = cx.new(|cx| {
            let id = cx.entity_id();
            cx.on_release(move |_: &mut Self, cx| {
                let variants = &mut cx.global_mut::<Images>().variants;
                if variants
                    .get(&key)
                    .is_some_and(|entry| entry.entity_id() == id)
                {
                    variants.remove(&key);
                }
            })
            .detach();
            Self {
                _source: source,
                target,
                animated: encoded.animated,
                encoded,
                decoder: None,
                current: None,
                next: None,
                delay: Duration::ZERO,
                due: None,
                waiting: HashSet::new(),
                error: None,
                work: None,
                wake: None,
            }
        });
        cx.global_mut::<Images>()
            .variants
            .insert(key, entity.downgrade());
        entity
    }
    fn decode(&mut self, cx: &mut Context<Self>) {
        if self.work.is_some() || self.error.is_some() {
            return;
        }
        let encoded = self.encoded.clone();
        let target = self.target;
        let decoder = self.decoder.take();
        let permits = cx.global::<Images>().decodes.clone();
        let worker = cx.background_executor().spawn(async move {
            let _permit = permits.acquire().await.expect("image permits remain open");
            let mut decoder = match decoder {
                Some(decoder) => decoder,
                None => encoded.decoder(target)?,
            };
            let frame = match decoder.next_frame()? {
                Some(frame) => frame,
                None => {
                    decoder = encoded.decoder(target)?;
                    decoder
                        .next_frame()?
                        .ok_or_else(|| ImageCacheError::Asset("Image has no frames".into()))?
                }
            };
            Ok::<_, ImageCacheError>((decoder, frame))
        });
        self.work = Some(cx.spawn(async move |this, cx| {
            let result = worker.await;
            let _ = this.update(cx, |this, cx| {
                this.work = None;
                match result {
                    Ok((decoder, DecodedFrame { image, delay })) => {
                        if this.animated {
                            this.decoder = Some(decoder);
                        }
                        let frame = ImagePixels::own(image, cx);
                        let delay = delay.max(Duration::from_millis(10));
                        if this.current.is_none() {
                            this.current = Some(frame);
                            this.delay = delay;
                        } else {
                            this.next = Some((frame, delay));
                        }
                    }
                    Err(error) => {
                        this.error = Some(error);
                        this.current = None;
                        this.next = None;
                        this.decoder = None;
                        this.wake = None;
                    }
                }
                for view in this.waiting.drain() {
                    App::notify(cx, view);
                }
            });
        }));
    }
    pub(super) fn frame(
        &mut self,
        view: EntityId,
        animate: bool,
        cx: &mut Context<Self>,
    ) -> Option<Result<ManagedImageFrame, ImageCacheError>> {
        if let Some(error) = &self.error {
            return Some(Err(error.clone()));
        }
        if self.current.is_none() {
            self.waiting.insert(view);
            self.decode(cx);
            return None;
        }
        if self.animated && animate {
            self.waiting.insert(view);
            let now = cx.background_executor().now();
            let due = *self.due.get_or_insert(now + self.delay);
            if now >= due
                && let Some((frame, delay)) = self.next.take()
            {
                self.current = Some(frame);
                self.delay = delay;
                self.due = Some(now + delay);
            }
            if self.next.is_none() {
                self.decode(cx);
            }
            if self.wake.is_none() {
                let delay = self
                    .due
                    .unwrap()
                    .saturating_duration_since(now)
                    .max(Duration::from_millis(10));
                self.wake = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(delay).await;
                    let _ = this.update(cx, |this, cx| {
                        this.wake = None;
                        for view in this.waiting.drain() {
                            App::notify(cx, view);
                        }
                    });
                }));
            }
        }
        self.current.clone().map(Ok)
    }
}
