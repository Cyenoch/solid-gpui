use std::io::Cursor;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;

use gif::{ColorOutput, DisposalMethod, MemoryLimit};
use gpui::{DevicePixels, ImageCacheError, ParsedSvg, RenderImage, SvgRenderer, SvgSize, size};
use image::imageops::{self, FilterType};
use image::{
    DynamicImage, GenericImageView, ImageBuffer, ImageFormat, ImageReader, Limits, RgbaImage,
};
use image_webp::WebPDecoder;
use smallvec::smallvec;

const MAX_SOURCE_PIXELS: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT_PIXELS: u64 = 16 * 1024 * 1024;
const MAX_DIMENSION: u32 = 32_768;
const MAX_DECODE_BYTES: usize = (MAX_SOURCE_PIXELS as usize) * 4;

type Bytes = Arc<[u8]>;
type GifReader = gif::Decoder<Cursor<Bytes>>;
type WebPReader = WebPDecoder<Cursor<Bytes>>;

pub(super) struct EncodedImage {
    bytes: Bytes,
    kind: EncodedKind,
    orientation: image::metadata::Orientation,
    pub size: (u32, u32),
    pub animated: bool,
    pub scalable: bool,
}

enum EncodedKind {
    Raster(ImageFormat),
    Svg {
        renderer: SvgRenderer,
        parsed: Arc<ParsedSvg>,
    },
}

pub(super) struct DecodedFrame {
    pub image: Arc<RenderImage>,
    pub delay: Duration,
}

pub(super) struct ImageDecoder {
    target: (u32, u32),
    state: DecoderState,
    finished: bool,
}

enum DecoderState {
    Static(StaticDecoder),
    Gif(Box<GifDecoder>),
    WebP(WebPDecoderState),
    Svg(SvgDecoder),
}

struct StaticDecoder {
    bytes: Bytes,
    format: ImageFormat,
    orientation: image::metadata::Orientation,
}

struct SvgDecoder {
    renderer: SvgRenderer,
    parsed: Arc<ParsedSvg>,
}

struct GifDecoder {
    reader: GifReader,
    canvas: RgbaImage,
}

struct WebPDecoderState {
    reader: WebPReader,
    buffer: Vec<u8>,
    alpha: bool,
    animated: bool,
}

impl EncodedImage {
    pub fn new(bytes: Vec<u8>, svg: SvgRenderer) -> Result<Self, ImageCacheError> {
        let bytes: Bytes = bytes.into();
        if let Ok(format) = image::guess_format(&bytes) {
            return Self::new_raster(bytes, format);
        }
        let parsed = Arc::new(
            svg.parse_svg(&bytes)
                .map_err(|error| ImageCacheError::Usvg(Arc::new(error)))?,
        );
        let intrinsic = parsed.size();
        let width = f32::from(intrinsic.width).ceil().max(1.0) as u32;
        let height = f32::from(intrinsic.height).ceil().max(1.0) as u32;
        validate_source_size(width, height)?;
        Ok(Self {
            bytes,
            kind: EncodedKind::Svg {
                renderer: svg,
                parsed,
            },
            orientation: image::metadata::Orientation::NoTransforms,
            size: (width, height),
            animated: false,
            scalable: true,
        })
    }
    fn new_raster(bytes: Bytes, format: ImageFormat) -> Result<Self, ImageCacheError> {
        let (width, height, animated) = match format {
            ImageFormat::Gif => {
                let reader = new_gif_reader(bytes.clone())?;
                let animated = gif_is_animated(bytes.clone())?;
                (
                    u32::from(reader.width()),
                    u32::from(reader.height()),
                    animated,
                )
            }
            ImageFormat::WebP => {
                let reader = new_webp_reader(bytes.clone())?;
                let (width, height) = reader.dimensions();
                (width, height, reader.is_animated())
            }
            _ => {
                let mut reader = ImageReader::with_format(Cursor::new(bytes.clone()), format);
                reader.limits(image_limits());
                let (width, height) = reader.into_dimensions().map_err(image_error)?;
                (width, height, false)
            }
        };
        validate_source_size(width, height)?;

        let orientation = if format == ImageFormat::Jpeg {
            jpeg_orientation(&bytes)
        } else if format == ImageFormat::Gif || animated {
            image::metadata::Orientation::NoTransforms
        } else {
            let mut reader = ImageReader::with_format(Cursor::new(bytes.clone()), format);
            reader.limits(image_limits());
            let mut decoder = reader.into_decoder().map_err(image_error)?;
            image::ImageDecoder::orientation(&mut decoder).map_err(image_error)?
        };
        let size = if orientation_swaps_axes(orientation) {
            (height, width)
        } else {
            (width, height)
        };

        Ok(Self {
            bytes,
            kind: EncodedKind::Raster(format),
            orientation,
            size,
            animated,
            scalable: false,
        })
    }

    pub fn decoder(self: &Arc<Self>, target: (u32, u32)) -> Result<ImageDecoder, ImageCacheError> {
        validate_target_size(target.0, target.1)?;
        let state = match &self.kind {
            EncodedKind::Svg { renderer, parsed } => DecoderState::Svg(SvgDecoder {
                renderer: renderer.clone(),
                parsed: parsed.clone(),
            }),
            EncodedKind::Raster(ImageFormat::Gif) => {
                let reader = new_gif_reader(self.bytes.clone())?;
                let canvas = RgbaImage::new(u32::from(reader.width()), u32::from(reader.height()));
                DecoderState::Gif(Box::new(GifDecoder { reader, canvas }))
            }
            EncodedKind::Raster(ImageFormat::WebP) if self.animated => {
                let reader = new_webp_reader(self.bytes.clone())?;
                let size = reader
                    .output_buffer_size()
                    .ok_or_else(|| decode_error("WebP output size overflows"))?;
                if size > MAX_DECODE_BYTES {
                    return Err(decode_error("WebP output exceeds the decode limit"));
                }
                let alpha = reader.has_alpha();
                let animated = reader.is_animated();
                DecoderState::WebP(WebPDecoderState {
                    reader,
                    buffer: vec![0; size],
                    alpha,
                    animated,
                })
            }
            EncodedKind::Raster(format) => DecoderState::Static(StaticDecoder {
                bytes: self.bytes.clone(),
                format: *format,
                orientation: self.orientation,
            }),
        };
        Ok(ImageDecoder {
            target,
            state,
            finished: false,
        })
    }
}

impl ImageDecoder {
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, ImageCacheError> {
        if self.finished {
            return Ok(None);
        }

        match &mut self.state {
            DecoderState::Static(decoder) => {
                self.finished = true;
                decoder.decode(self.target).map(Some)
            }
            DecoderState::Svg(decoder) => {
                self.finished = true;
                decoder.decode(self.target).map(Some)
            }
            DecoderState::Gif(decoder) => decoder.next_frame(self.target),
            DecoderState::WebP(decoder) => {
                let frame = decoder.next_frame(self.target)?;
                if !decoder.animated || frame.is_none() {
                    self.finished = true;
                }
                Ok(frame)
            }
        }
    }
}

impl StaticDecoder {
    fn decode(&mut self, target: (u32, u32)) -> Result<DecodedFrame, ImageCacheError> {
        let image = if self.format == ImageFormat::Jpeg {
            decode_jpeg(self.bytes.clone(), target, self.orientation)?
        } else {
            // These formats have no decoder-native reduction in image-rs. Source dimensions and
            // decoder allocations are bounded, but their transient decode peak is source-sized.
            let mut reader = ImageReader::with_format(Cursor::new(self.bytes.clone()), self.format);
            reader.limits(image_limits());
            let mut image = reader.decode().map_err(image_error)?;
            image.apply_orientation(self.orientation);
            resize_exact(image, target)
        };
        Ok(decoded_frame(image, Duration::ZERO))
    }
}

impl SvgDecoder {
    fn decode(&self, target: (u32, u32)) -> Result<DecodedFrame, ImageCacheError> {
        let render = self
            .renderer
            .render_parsed(
                &self.parsed,
                SvgSize::ExactSize(size(
                    DevicePixels::from(target.0),
                    DevicePixels::from(target.1),
                )),
            )
            .map_err(|error| ImageCacheError::Usvg(Arc::new(error)))?;
        Ok(DecodedFrame {
            image: render,
            delay: Duration::ZERO,
        })
    }
}

impl GifDecoder {
    fn next_frame(&mut self, target: (u32, u32)) -> Result<Option<DecodedFrame>, ImageCacheError> {
        let Some(frame) = self.reader.read_next_frame().map_err(external_error)? else {
            return Ok(None);
        };
        let left = u32::from(frame.left);
        let top = u32::from(frame.top);
        let width = u32::from(frame.width);
        let height = u32::from(frame.height);
        if left
            .checked_add(width)
            .is_none_or(|right| right > self.canvas.width())
            || top
                .checked_add(height)
                .is_none_or(|bottom| bottom > self.canvas.height())
        {
            return Err(decode_error("GIF frame lies outside its canvas"));
        }

        let mut output = self.canvas.clone();
        for y in 0..height {
            for x in 0..width {
                let source = ((y * width + x) * 4) as usize;
                let pixel = &frame.buffer[source..source + 4];
                if pixel[3] != 0 {
                    output.put_pixel(
                        left + x,
                        top + y,
                        image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]),
                    );
                }
            }
        }

        match frame.dispose {
            DisposalMethod::Any | DisposalMethod::Keep => self.canvas.clone_from(&output),
            DisposalMethod::Background => {
                self.canvas.clone_from(&output);
                for y in top..top + height {
                    for x in left..left + width {
                        self.canvas.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                    }
                }
            }
            DisposalMethod::Previous => {}
        }

        let delay = Duration::from_millis(u64::from(frame.delay) * 10);
        Ok(Some(decoded_frame(
            resize_exact(DynamicImage::ImageRgba8(output), target),
            delay,
        )))
    }
}

impl WebPDecoderState {
    fn next_frame(&mut self, target: (u32, u32)) -> Result<Option<DecodedFrame>, ImageCacheError> {
        let delay = if self.animated {
            match self.reader.read_frame(&mut self.buffer) {
                Ok(delay) => Duration::from_millis(u64::from(delay)),
                Err(image_webp::DecodingError::NoMoreFrames) => return Ok(None),
                Err(error) => return Err(external_error(error)),
            }
        } else {
            self.reader
                .read_image(&mut self.buffer)
                .map_err(external_error)?;
            Duration::ZERO
        };
        let (width, height) = self.reader.dimensions();
        let image = if self.alpha {
            let source = ImageBuffer::from_raw(width, height, self.buffer.as_slice())
                .ok_or_else(|| decode_error("WebP decoder returned an invalid buffer"))?;
            DynamicImage::ImageRgba8(imageops::resize(
                &source,
                target.0,
                target.1,
                FilterType::Lanczos3,
            ))
        } else {
            let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);
            for rgb in self.buffer.as_chunks::<3>().0 {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
            let image = RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| decode_error("WebP decoder returned an invalid buffer"))?;
            resize_exact(DynamicImage::ImageRgba8(image), target)
        };
        Ok(Some(decoded_frame(image, delay)))
    }
}

fn decode_jpeg(
    bytes: Bytes,
    target: (u32, u32),
    orientation: image::metadata::Orientation,
) -> Result<DynamicImage, ImageCacheError> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    decoder.set_max_decoding_buffer_size(MAX_DECODE_BYTES);
    decoder.read_info().map_err(external_error)?;
    let source = decoder
        .info()
        .ok_or_else(|| decode_error("JPEG has no frame information"))?;
    let requested = if orientation_swaps_axes(orientation) {
        (target.1, target.0)
    } else {
        target
    };
    if matches!(
        source.coding_process,
        jpeg_decoder::CodingProcess::DctSequential | jpeg_decoder::CodingProcess::DctProgressive
    ) {
        decoder
            .scale(
                requested.0.min(u16::MAX as u32) as u16,
                requested.1.min(u16::MAX as u32) as u16,
            )
            .map_err(external_error)?;
    }
    let pixels = decoder.decode().map_err(external_error)?;
    let info = decoder
        .info()
        .ok_or_else(|| decode_error("JPEG lost frame information"))?;
    let decoded_dimensions = (u32::from(info.width), u32::from(info.height));
    let mut rgba = Vec::with_capacity(usize::from(info.width) * usize::from(info.height) * 4);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => {
            for &luma in &pixels {
                rgba.extend_from_slice(&[luma, luma, luma, 255]);
            }
        }
        jpeg_decoder::PixelFormat::L16 => {
            for luma in pixels.as_chunks::<2>().0 {
                let luma = u16::from_ne_bytes([luma[0], luma[1]]) >> 8;
                rgba.extend_from_slice(&[luma as u8, luma as u8, luma as u8, 255]);
            }
        }
        jpeg_decoder::PixelFormat::RGB24 => {
            for rgb in pixels.as_chunks::<3>().0 {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            for cmyk in pixels.as_chunks::<4>().0 {
                let key = 1.0 - f32::from(cmyk[3]) / 255.0;
                rgba.extend_from_slice(&[
                    ((255.0 - f32::from(cmyk[0])) * key) as u8,
                    ((255.0 - f32::from(cmyk[1])) * key) as u8,
                    ((255.0 - f32::from(cmyk[2])) * key) as u8,
                    255,
                ]);
            }
        }
    }
    let image = RgbaImage::from_raw(decoded_dimensions.0, decoded_dimensions.1, rgba)
        .ok_or_else(|| decode_error("JPEG decoder returned an invalid buffer"))?;
    let mut image = DynamicImage::ImageRgba8(image);
    image.apply_orientation(orientation);
    Ok(resize_exact(image, target))
}

fn resize_exact(image: DynamicImage, target: (u32, u32)) -> DynamicImage {
    if image.dimensions() == target {
        image
    } else {
        image.resize_exact(target.0, target.1, FilterType::Lanczos3)
    }
}

fn decoded_frame(image: DynamicImage, delay: Duration) -> DecodedFrame {
    let mut buffer = image.into_rgba8();
    for pixel in buffer.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let frame = image::Frame::new(buffer);
    DecodedFrame {
        image: Arc::new(RenderImage::new(smallvec![frame])),
        delay,
    }
}

fn new_gif_reader(bytes: Bytes) -> Result<GifReader, ImageCacheError> {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(ColorOutput::RGBA);
    options.set_memory_limit(MemoryLimit::Bytes(
        NonZeroU64::new(MAX_DECODE_BYTES as u64).expect("non-zero GIF memory limit"),
    ));
    options.check_frame_consistency(true);
    options
        .read_info(Cursor::new(bytes))
        .map_err(external_error)
}

fn gif_is_animated(bytes: Bytes) -> Result<bool, ImageCacheError> {
    let mut options = gif::DecodeOptions::new();
    options.skip_frame_decoding(true);
    options.set_memory_limit(MemoryLimit::Bytes(
        NonZeroU64::new(MAX_DECODE_BYTES as u64).expect("non-zero GIF memory limit"),
    ));
    let mut reader = options
        .read_info(Cursor::new(bytes))
        .map_err(external_error)?;
    if reader.read_next_frame().map_err(external_error)?.is_none() {
        return Err(decode_error("GIF contains no frames"));
    }
    Ok(reader.read_next_frame().map_err(external_error)?.is_some())
}

fn new_webp_reader(bytes: Bytes) -> Result<WebPReader, ImageCacheError> {
    let mut reader = WebPDecoder::new(Cursor::new(bytes)).map_err(external_error)?;
    reader.set_memory_limit(MAX_DECODE_BYTES);
    Ok(reader)
}

fn image_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_BYTES as u64);
    limits
}

fn validate_source_size(width: u32, height: u32) -> Result<(), ImageCacheError> {
    if width == 0 || height == 0 {
        return Err(decode_error("image dimensions must be non-zero"));
    }
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(decode_error("image dimension exceeds 32768 pixels"));
    }
    if u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS {
        return Err(decode_error("image exceeds the 64 megapixel source limit"));
    }
    Ok(())
}

fn validate_target_size(width: u32, height: u32) -> Result<(), ImageCacheError> {
    if width == 0 || height == 0 {
        return Err(decode_error("target dimensions must be non-zero"));
    }
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(decode_error("target dimension exceeds 32768 pixels"));
    }
    if u64::from(width) * u64::from(height) > MAX_OUTPUT_PIXELS {
        return Err(decode_error("target exceeds the 16 megapixel output limit"));
    }
    Ok(())
}

fn orientation_swaps_axes(orientation: image::metadata::Orientation) -> bool {
    matches!(
        orientation,
        image::metadata::Orientation::Rotate90
            | image::metadata::Orientation::Rotate270
            | image::metadata::Orientation::Rotate90FlipH
            | image::metadata::Orientation::Rotate270FlipH
    )
}

fn jpeg_orientation(bytes: &[u8]) -> image::metadata::Orientation {
    let mut offset = 2usize;
    while bytes.get(..2) == Some(&[0xff, 0xd8]) && offset + 4 <= bytes.len() {
        if bytes[offset] != 0xff {
            break;
        }
        while bytes.get(offset) == Some(&0xff) {
            offset += 1;
        }
        let Some(&marker) = bytes.get(offset) else {
            break;
        };
        offset += 1;
        if marker == 0xda || marker == 0xd9 {
            break;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let Some(length_bytes) = bytes.get(offset..offset + 2) else {
            break;
        };
        let length = usize::from(u16::from_be_bytes([length_bytes[0], length_bytes[1]]));
        if length < 2
            || offset
                .checked_add(length)
                .is_none_or(|end| end > bytes.len())
        {
            break;
        }
        if marker == 0xe1 {
            let payload = &bytes[offset + 2..offset + length];
            if let Some(tiff) = payload.strip_prefix(b"Exif\0\0")
                && let Some(orientation) = image::metadata::Orientation::from_exif_chunk(tiff)
            {
                return orientation;
            }
        }
        offset += length;
    }
    image::metadata::Orientation::NoTransforms
}

fn decode_error(message: impl Into<String>) -> ImageCacheError {
    ImageCacheError::Asset(message.into().into())
}

fn image_error(error: image::ImageError) -> ImageCacheError {
    ImageCacheError::Image(Arc::new(error))
}

fn external_error(error: impl std::fmt::Display) -> ImageCacheError {
    decode_error(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, ImageEncoder};

    #[test]
    fn static_png_orientation_survives_target_decode() {
        let source = image::RgbImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                image::Rgb([255, 0, 0])
            } else {
                image::Rgb([0, 255, 0])
            }
        });
        let mut bytes = Vec::new();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        encoder
            .set_exif_metadata(
                b"II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0".to_vec(),
            )
            .unwrap();
        encoder
            .write_image(source.as_raw(), 2, 1, image::ExtendedColorType::Rgb8)
            .unwrap();
        let encoded = Arc::new(EncodedImage::new(bytes, SvgRenderer::new(Arc::new(()))).unwrap());
        assert_eq!(encoded.size, (1, 2));
        let frame = encoded
            .decoder((1, 2))
            .unwrap()
            .next_frame()
            .unwrap()
            .unwrap();
        assert_eq!(
            frame.image.as_bytes(0).unwrap(),
            &[0, 0, 255, 255, 0, 255, 0, 255]
        );
    }

    #[test]
    fn dimension_limits_reject_before_allocation() {
        assert!(validate_source_size(MAX_DIMENSION + 1, 1).is_err());
        assert!(validate_source_size(8193, 8192).is_err());
        assert!(validate_target_size(4097, 4096).is_err());
        assert!(validate_target_size(4096, 4096).is_ok());
    }

    #[test]
    fn jpeg_orientation_parser_swaps_intrinsic_axes() {
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xff, 0xe1, 0, 34];
        jpeg.extend_from_slice(
            b"Exif\0\0II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0",
        );
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        assert_eq!(
            jpeg_orientation(&jpeg),
            image::metadata::Orientation::Rotate90
        );
        assert!(orientation_swaps_axes(jpeg_orientation(&jpeg)));
    }

    #[test]
    fn jpeg_decoder_scales_and_emits_target_sized_bgra() {
        let source = image::RgbImage::from_fn(64, 32, |x, _| image::Rgb([x as u8, 2, 3]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 90)
            .write_image(source.as_raw(), 64, 32, image::ExtendedColorType::Rgb8)
            .unwrap();

        let mut native = jpeg_decoder::Decoder::new(Cursor::new(bytes.as_slice()));
        native.read_info().unwrap();
        let scaled = native.scale(13, 7).unwrap();
        assert!(scaled.0 < 64 && scaled.1 < 32);

        let image = decode_jpeg(
            bytes.into(),
            (13, 7),
            image::metadata::Orientation::NoTransforms,
        )
        .unwrap();
        assert_eq!(image.dimensions(), (13, 7));
        let frame = decoded_frame(image, Duration::ZERO);
        assert_eq!(frame.image.as_bytes(0).unwrap().len(), 13 * 7 * 4);
    }

    #[test]
    fn gif_stream_preserves_order_disposal_and_target_size() {
        use std::borrow::Cow;

        let palette = [255, 0, 0, 0, 255, 0, 0, 0, 255];
        let mut bytes = Vec::new();
        {
            let mut encoder = gif::Encoder::new(&mut bytes, 2, 1, &palette).unwrap();
            let first = gif::Frame {
                width: 2,
                height: 1,
                delay: 2,
                dispose: DisposalMethod::Background,
                buffer: Cow::Borrowed(&[0, 0]),
                ..Default::default()
            };
            encoder.write_frame(&first).unwrap();

            let second = gif::Frame {
                left: 1,
                width: 1,
                height: 1,
                delay: 3,
                buffer: Cow::Borrowed(&[1]),
                ..Default::default()
            };
            encoder.write_frame(&second).unwrap();
        }

        let reader = new_gif_reader(bytes.into()).unwrap();
        let mut decoder = GifDecoder {
            reader,
            canvas: RgbaImage::new(2, 1),
        };
        let first = decoder.next_frame((4, 2)).unwrap().unwrap();
        assert_eq!(first.delay, Duration::from_millis(20));
        assert_eq!(first.image.as_bytes(0).unwrap().len(), 4 * 2 * 4);
        let second = decoder.next_frame((2, 1)).unwrap().unwrap();
        assert_eq!(second.delay, Duration::from_millis(30));
        assert_eq!(
            second.image.as_bytes(0).unwrap(),
            &[0, 0, 0, 0, 0, 255, 0, 255]
        );
        assert!(decoder.next_frame((2, 1)).unwrap().is_none());
    }
}
