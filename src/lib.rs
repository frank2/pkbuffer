//! [PKBuffer](https://github.com/frank2/pkbuffer) is a library built for arbitrary casting of data structures
//! onto segments of memory! This includes sections of unowned memory, such as examining the headers of a
//! currently running executable. It creates an interface for reading and writing data structures to an
//! arbitrary buffer of bytes.
//!
//! For example:
//! ```rust
//! use pkbuffer::{Buffer, VecBuffer, Castable};
//!
//! #[repr(packed)]
//! #[derive(Copy, Clone, Castable)]
//! struct Object {
//!    byte: u8,
//!    word: u16,
//!    dword: u32,
//! }
//!
//! let mut buffer = VecBuffer::with_initial_size(std::mem::size_of::<Object>());
//! let object = buffer.get_mut_ref::<Object>(0).unwrap();
//! object.byte = 0x01;
//! object.word = 0x0302;
//! object.dword = 0x07060504;
//!
//! assert_eq!(buffer, [1,2,3,4,5,6,7]);
//! ```
//!
//! Objects retrieved from [`Buffer`](Buffer) objects must implement the [`Castable`](castable::Castable)
//! trait. This trait ensures that a series of attributes are applied to the object. For
//! convenience, a [derive macro](pkbuffer_derive::Castable) is provided.
//!
//! Buffer objects are derived from the [`Buffer`](Buffer) trait. This trait
//! implements much functionality of slice objects as well as data casting
//! abilities of the derived Buffer objects.
//!
//! Buffer objects comes in two forms: *pointer form* ([`PtrBuffer`](PtrBuffer)) and
//! *allocated form* ([`VecBuffer`](VecBuffer)). Each of these structures come
//! in handy for different reasons. [`PtrBuffer`](PtrBuffer) is useful on unowned data
//! such as arbitrary locations in memory, whereas [`VecBuffer`](VecBuffer)'s
//! utility comes from being able to manipulate the underlying owned data.
//!
//! [`VecBuffer`](VecBuffer)s are handy for creating a brand-new buffer of objects.
//!
//! ```rust
//! use pkbuffer::{Buffer, VecBuffer};
//!
//! let mut buffer = VecBuffer::new();
//! buffer.append_ref::<u8>(&0x1);
//! buffer.append_ref::<u16>(&0x0302);
//! buffer.append_ref::<u32>(&0x07060504);
//! assert_eq!(buffer, [1,2,3,4,5,6,7]);
//! ```

#[cfg(test)]
mod tests;

mod buffer;
pub use buffer::*;

mod castable;
pub use castable::*;

mod ptr;
pub use ptr::*;

mod vec;
pub use vec::*;

pub use pkbuffer_derive::*;

/// Errors produced by the library.
#[derive(Debug)]
pub enum Error {
    /// An error produced by [`std::io::Error`](std::io::Error).
    IoError(std::io::Error),
    /// The operation went out of bounds.
    ///
    /// The first arg represents the current boundary, the second arg
    /// represents the out-of-bounds argument.
    OutOfBounds(usize,usize),
    /// The operation produced an invalid pointer. Argument is the pointer
    /// in question.
    InvalidPointer(*const u8),
    /// The alignment of the given operation is off. The first arg
    /// represents the expected alignment, the second argument represents
    /// the alignment of the given object relative to the expected alignment.
    BadAlignment(usize,usize),
    /// The type is zero-sized.
    ZeroSizedType,
    /// The sizes didn't match. The first arg represents the expected size,
    /// the second arg represents the received size.
    SizeMismatch(usize,usize),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(io) => write!(f, "i/o error: {}", io.to_string()),
            Self::OutOfBounds(expected,got) => write!(f, "out of bounds: boundary is {:#x}, got {:#x} instead", expected, got),
            Self::InvalidPointer(ptr) => write!(f, "invalid pointer: {:p}", ptr),
            Self::BadAlignment(expected,got) => write!(f, "bad alignment: expected {}-byte alignment, but alignment is off by {}", expected, got),
            Self::ZeroSizedType => write!(f, "zero sized type"),
            Self::SizeMismatch(expected,got) => write!(f, "size mismatch: the two types differed in size, expected {}, got {}", expected, got),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(ref e) => Some(e),
            _ => None,
        }
    }
}
impl std::convert::From<std::io::Error> for Error {
    fn from(io_err: std::io::Error) -> Self {
        Self::IoError(io_err)
    }
}
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

/// Convert the given reference of type ```T``` to a [`u8`](u8) [slice](slice).
pub fn ref_to_bytes<T: Castable>(data: &T) -> Result<&[u8], Error> {
    if std::mem::size_of::<T>() == 0 { Ok(&[]) }
    else { slice_ref_to_bytes::<T>(std::slice::from_ref(data)) }
}

/// Convert the given slice reference of type ```T``` to a [`u8`](u8) [slice](slice).
pub fn slice_ref_to_bytes<T: Castable>(data: &[T]) -> Result<&[u8], Error> {
    if std::mem::size_of::<T>() == 0 {
        Err(Error::ZeroSizedType)
    }
    else {
        Ok(unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) })
    }
}

/// Convert the given reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
pub fn ref_to_mut_bytes<T: Castable>(data: &mut T) -> Result<&mut [u8], Error> {
    if std::mem::size_of::<T>() == 0 { Ok(&mut []) }
    else { slice_ref_to_mut_bytes::<T>(std::slice::from_mut(data)) }
}

/// Convert the given slice reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
pub fn slice_ref_to_mut_bytes<T: Castable>(data: &mut [T]) -> Result<&mut [u8], Error> {
    if std::mem::size_of::<T>() == 0 {
        Err(Error::ZeroSizedType)
    }
    else {
        Ok(unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, std::mem::size_of_val(data)) })
    }
}

/// Cast type `&T` from a [`u8`](u8) slice.
pub fn bytes_to_ref<T: Castable>(bytes: &[u8]) -> Result<&T, Error> {
    if bytes.len() != std::mem::size_of::<T>() {
        Err(Error::SizeMismatch(bytes.len(), std::mem::size_of::<T>()))
    }
    else if (bytes.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
        Err(Error::BadAlignment(std::mem::align_of::<T>(), (bytes.as_ptr() as usize) % std::mem::align_of::<T>()))
    }
    else {
        Ok(unsafe { &*(bytes.as_ptr() as *const T) })
    }
}

/// Cast type `&mut T` from a mutable [`u8`](u8) slice.
pub fn bytes_to_mut_ref<T: Castable>(bytes: &mut [u8]) -> Result<&mut T, Error> {
    if bytes.len() != std::mem::size_of::<T>() {
        Err(Error::SizeMismatch(bytes.len(), std::mem::size_of::<T>()))
    }
    else if (bytes.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
        Err(Error::BadAlignment(std::mem::align_of::<T>(), (bytes.as_ptr() as usize) % std::mem::align_of::<T>()))
    }
    else {
        Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut T) })
    }
}
