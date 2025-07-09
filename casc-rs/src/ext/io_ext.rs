use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

pub trait ReadExt: Read + Seek {
    fn read_chars(&mut self, count: usize) -> io::Result<Vec<char>>;

    fn peek_byte(&mut self) -> io::Result<u8>;
}

impl<T> ReadExt for T
where
    T: Read + Seek,
{
    /// Reads up to `count` UTF-8 characters from the reader, advancing only by the bytes needed.
    fn read_chars(&mut self, count: usize) -> io::Result<Vec<char>> {
        let mut chars = Vec::with_capacity(count);
        let mut buf = [0u8; 4];

        while chars.len() < count {
            let mut first = [0u8; 1];
            if self.read(&mut first)? == 0 {
                break;
            }
            buf[0] = first[0];

            let char_len = match first[0] {
                0x00..=0x7F => 1,
                0xC0..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF7 => 4,
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8")),
            };

            for i in 1..char_len {
                if self.read(&mut buf[i..i + 1])? == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpected EOF",
                    ));
                }
            }
            let s = std::str::from_utf8(&buf[..char_len])
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
            let ch = s.chars().next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Failed to convert to char")
            })?;
            chars.push(ch);
        }
        Ok(chars)
    }
    /// Peeks a single byte from the reader without advancing its position.
    fn peek_byte(&mut self) -> io::Result<u8> {
        let pos = self.stream_position()?;
        let mut buf = [0u8; 1];
        let n = self.read(&mut buf)?;
        self.seek(SeekFrom::Start(pos))?;
        if n == 0 {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))
        } else {
            Ok(buf[0])
        }
    }
}

/// A trait that reads arrays from any `Read` type.
pub trait ArrayReadExt: Read {
    /// Reads an array of `R` with the given length.
    fn read_array<R>(&mut self, length: usize) -> io::Result<Vec<R>>
    where
        R: Copy + 'static;

    /// Reads an array of 'u8' until EOF.
    fn read_array_to_end(&mut self) -> io::Result<Vec<u8>>;
}

impl<T> ArrayReadExt for T
where
    T: Read,
{
    fn read_array<R>(&mut self, length: usize) -> Result<Vec<R>, io::Error>
    where
        R: Copy + 'static,
    {
        let mut result: Vec<std::mem::MaybeUninit<R>> = Vec::new();

        result
            .try_reserve_exact(length)
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))?;
        result.resize(length, std::mem::MaybeUninit::<R>::zeroed());

        let slice = result.as_mut_slice();

        // SAFETY: We ensure that the size of each element correctly matches the size in bytes.
        let slice = unsafe {
            std::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut u8,
                slice.len() * size_of::<R>(),
            )
        };

        self.read_exact(slice)?;

        let mut result = std::mem::ManuallyDrop::new(result);

        let ptr = result.as_mut_ptr();
        let len = result.len();
        let cap = result.capacity();

        // SAFETY: The source data was a Vec<> and MaybeUninit always has the same memory layout as T.
        Ok(unsafe { Vec::from_raw_parts(ptr as *mut R, len, cap) })
    }

    fn read_array_to_end(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut result: Vec<u8> = Vec::new();

        self.read_to_end(&mut result)?;

        Ok(result)
    }
}

/// Utility methods for working with seekable streams.
pub trait SeekExt: Seek {
    /// Skips over the given number of bytes from the current position.
    fn skip<P: Copy + 'static>(&mut self, size: P) -> io::Result<u64>
    where
        u64: TryFrom<P>;
}

impl<T> SeekExt for T
where
    T: Seek,
{
    fn skip<P: Copy + 'static>(&mut self, size: P) -> io::Result<u64>
    where
        u64: TryFrom<P>,
    {
        let size = u64::try_from(size).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        self.seek(SeekFrom::Current(size as i64))
    }
}

/// A trait that reads structs from `Read` sources.
pub trait StructReadExt: Read {
    /// Reads the type from the reader and advances the stream.
    fn read_struct<S: Copy + 'static>(&mut self) -> io::Result<S>;
}

impl<T> StructReadExt for T
where
    T: Read,
{
    fn read_struct<S: Copy + 'static>(&mut self) -> Result<S, io::Error> {
        let mut result = std::mem::MaybeUninit::<S>::zeroed();

        // SAFETY: This slice has the same length as T, and T is always Copy.
        let slice = unsafe {
            std::slice::from_raw_parts_mut(result.as_mut_ptr() as *mut u8, size_of::<S>())
        };

        self.read_exact(slice)?;

        // SAFETY: As long as `read_exact` is safe, we can assume that the full data was initialized.
        Ok(unsafe { result.assume_init() })
    }
}
