//! Images are stored as URLs. That often means they are encoded as Data-URLs, but that may not
//! always be the case. Each image object comes with an ID that changes whenever the image is
//! modified. IDs are unique across different images. ID 0 is never used so it can be used as an
//! initial state to refresh state at the beginning.

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use base64::{self, DecodeError, STANDARD};
use imagelib::{guess_format, ImageFormat};

static LAST_IMAGE_ID: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Debug, Clone)]
pub struct Image {
    url: String,
    id: usize,
    format: Option<ImageFormat>,
}

impl PartialEq for Image {
    fn eq(&self, other: &Image) -> bool {
        self.url == other.url
    }
}

impl Default for Image {
    fn default() -> Image {
        Image::new(&[])
    }
}

impl<D: AsRef<[u8]>> From<D> for Image {
    fn from(d: D) -> Self {
        Image::new(d.as_ref())
    }
}

impl Image {
    pub fn new(data: &[u8]) -> Self {
        let mut image = Image {
            url: String::new(),
            id: 0,
            format: None,
        };
        image.modify(data);
        image
    }

    pub fn from_file<P, B>(path: P, mut buf: B) -> io::Result<Image>
    where
        P: AsRef<Path>,
        B: AsMut<Vec<u8>>,
    {
        let buf = buf.as_mut();
        buf.clear();
        BufReader::new(File::open(path)?).read_to_end(buf)?;
        Ok(Image::new(buf))
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[inline]
    pub fn format(&self) -> Option<ImageFormat> {
        self.format
    }

    #[inline]
    pub fn mime_type(&self) -> Option<&'static str> {
        self.format.map(|format| {
            use self::ImageFormat::*;
            match format {
                BMP => "image/bmp",
                GIF => "image/gif",
                HDR => "image/vnd.radiance",
                ICO => "image/x-icon",
                JPEG => "image/jpeg",
                PNG => "image/png",
                PPM => "image/x-portable-pixmap",
                TGA => "image/x-tga",
                TIFF => "image/tiff",
                WEBP => "image/webp",
            }
        })
    }

    #[inline]
    pub fn file_extension(&self) -> Option<&'static str> {
        self.format.map(|format| {
            use self::ImageFormat::*;
            match format {
                BMP => "bmp",
                GIF => "gif",
                HDR => "hdr",
                ICO => "ico",
                JPEG => "jpg",
                PNG => "png",
                PPM => "ppm",
                TGA => "tga",
                TIFF => "tiff",
                WEBP => "webp",
            }
        })
    }

    #[inline]
    pub fn decode_data(&self, buf: &mut Vec<u8>) -> Result<(), DecodeError> {
        let url = self.url();
        buf.clear();
        if url.starts_with("data:") {
            let url = &url["data:".len()..];
            if let Some(index) = url.find(";base64,") {
                let src = &url[index + ";base64,".len()..];
                base64::decode_config_buf(src, base64::STANDARD, buf)?;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn check_for_change(&self, old_id: &mut usize) -> Option<&str> {
        if *old_id != self.id {
            *old_id = self.id;
            Some(self.url())
        } else {
            None
        }
    }

    pub fn modify(&mut self, data: &[u8]) {
        self.id = LAST_IMAGE_ID.fetch_add(1, Ordering::SeqCst) + 1;
        self.url.clear();

        if !data.is_empty() {
            self.url.push_str("data:");
            self.format = guess_format(data).ok();
            if let Some(mime_type) = self.mime_type() {
                self.url.push_str(mime_type);
            }
            self.url.push_str(";base64,");
            base64::encode_config_buf(data, STANDARD, &mut self.url);
        } else {
            self.format = None;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.url.is_empty()
    }
}
