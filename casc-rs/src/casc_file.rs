use crate::block_table::block_table_encoder_type::BlockTableEncoderType;
use crate::casc_file_span::CascFileSpan;
use flate2::read::ZlibDecoder;
use std::{
    fs::File,
    io::{self, Error, ErrorKind, Read, Seek, SeekFrom},
};

/// This struct manages reading, seeking, and caching data from multiple file spans,

/// handling decompression and decryption as needed.

pub struct CascFile {
    /// The spans that make up the file.
    pub spans: Vec<CascFileSpan<File>>,
    /// The total size of the file.
    internal_size: u64,
    /// The current read position within the file.
    internal_position: u64,
    /// Whether the stream is open.
    is_open: bool,
    /// Optional cache for read data.
    cache: Option<Vec<u8>>,
    /// The start position of the cache.
    cache_start_position: u64,
    /// The end position of the cache.
    cache_end_position: u64,
}

impl CascFile {
    /// Creates a new `File` from the given spans and size.

    pub(crate) fn new(spans: Vec<CascFileSpan<File>>, size: u64) -> Self {
        CascFile {
            spans,
            internal_size: size,
            internal_position: 0,
            is_open: true,
            cache: None,
            cache_start_position: 0,
            cache_end_position: 0,
        }
    }

    /// Returns the total size of the file.
    pub fn size(&self) -> u64 {
        self.internal_size
    }
}

impl Read for CascFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.is_open {
            return Err(Error::new(ErrorKind::Other, "Stream is closed"));
        }
        let mut read_start_pos = self.internal_position;
        if read_start_pos >= self.internal_size {
            return Ok(0);
        }
        let mut to_read = buf.len();
        let mut consumed = 0;
        let mut offset = 0;

        while to_read > 0 {
            let cache_available = self.cache_end_position.saturating_sub(read_start_pos);
            if let Some(ref cache) = self.cache {
                if cache_available > 0 {
                    if self.cache_start_position <= read_start_pos
                        && self.cache_end_position > read_start_pos
                    {
                        let p = (read_start_pos - self.cache_start_position) as usize;
                        let buf_available = buf.len().saturating_sub(offset);
                        let n = std::cmp::min(
                            to_read,
                            std::cmp::min(cache_available as usize, buf_available),
                        );
                        buf[offset..offset + n].copy_from_slice(&cache[p..p + n]);
                        to_read -= n;
                        self.seek(SeekFrom::Current(n as i64))?;
                        offset += n;
                        consumed += n;
                    }
                }
            }

            if to_read == 0 {
                break;
            }
            read_start_pos = self.internal_position;
            if read_start_pos >= self.internal_size {
                break;
            }
            // Find next span and frame
            let span = self
                .spans
                .iter_mut()
                .find(|x| {
                    read_start_pos >= x.virtual_start_offset
                        && read_start_pos < x.virtual_end_offset
                })
                .ok_or_else(|| Error::other("Span not found"))?;
            let frame = span
                .frames
                .iter_mut()
                .find(|x| {
                    read_start_pos >= x.virtual_start_offset
                        && read_start_pos < x.virtual_end_offset
                })
                .ok_or_else(|| Error::other("Frame not found"))?;
            // Lock the span reader
            let mut span_reader = span.span_reader.try_clone()?;
            span_reader.seek(SeekFrom::Start(frame.archive_offset))?;
            self.cache_start_position = frame.virtual_start_offset;
            self.cache_end_position = self.cache_start_position + frame.content_size as u64;
            let mut type_buf = [0u8; 1];
            span_reader.read_exact(&mut type_buf)?;
            let block_type = BlockTableEncoderType::from(type_buf[0]);
            self.cache = Some(match block_type {
                BlockTableEncoderType::Raw => {
                    let mut cache = vec![0u8; frame.content_size as usize];
                    span_reader.read_exact(&mut cache)?;
                    cache
                }
                BlockTableEncoderType::ZLib => {
                    let mut encoded = vec![0u8; frame.encoded_size as usize - 1];
                    span_reader.read_exact(&mut encoded)?;
                    let mut decoder = ZlibDecoder::new(&encoded[..]);
                    let mut cache = Vec::with_capacity(frame.content_size as usize);
                    decoder.read_to_end(&mut cache)?;
                    cache
                }
                _ => return Err(Error::new(ErrorKind::Other, "Unsupported Block Table Type")),
            });
        }
        Ok(consumed)
    }
}

impl Seek for CascFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.internal_position = offset,
            SeekFrom::Current(offset) => {
                self.internal_position = (self.internal_position as i64 + offset) as u64
            }
            SeekFrom::End(offset) => {
                self.internal_position = (self.internal_size as i64 - offset) as u64
            }
        }
        Ok(self.internal_position)
    }
}
