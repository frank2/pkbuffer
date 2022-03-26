//! [PKBuffer](https://github.com/frank2/pkbuffer) is a library built for arbitrary casting of data structures
//! onto segments of memory! This includes sections of unowned memory, such as examining the headers of a
//! currently running executable. It creates an interface for reading and writing data structures to an
//! arbitrary buffer of bytes.
//!
//! For example:
//! ```rust
//! use pkbuffer::{Buffer, VecBuffer, Pod, Zeroable};
//!
//! #[repr(packed)]
//! #[derive(Copy, Clone)]
//! struct Object {
//!    byte: u8,
//!    word: u16,
//!    dword: u32,
//! }
//! unsafe impl Pod for Object { }
//! unsafe impl Zeroable for Object {
//!    fn zeroed() -> Self { Self { byte: 0, word: 0, dword: 0 } }
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
//! Backing the conversion of objects and slices is the [bytemuck](bytemuck) library.
//! To yank objects out of a given buffer object, one must implement the [`Pod`](Pod)
//! trait on them. This trait has been exported from [bytemuck](bytemuck) for convenience.
//! You might also be interested in [bytemuck_derive](https://crates.io/crate/bytemuck_derive).
//! See the [bytemuck](bytemuck) documentation for more details.
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

pub use bytemuck::{Pod, Zeroable};
use bytemuck::*;

/// Errors produced by the library.
#[derive(Debug)]
pub enum Error {
    /// An error produced by [`std::io::Error`](std::io::Error).
    IoError(std::io::Error),
    /// An error produced by the [bytemuck](bytemuck) library.
    BytemuckError(PodCastError),
    /// The operation went out of bounds.
    ///
    /// The first arg represents the current boundary, the second arg
    /// represents the out-of-bounds argument.
    OutOfBounds(usize,usize),
    /// The operation produced an invalid pointer. Argument is the pointer
    /// in question.
    InvalidPointer(*const u8),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(io) => write!(f, "i/o error: {}", io.to_string()),
            Self::BytemuckError(bytemuck) => write!(f, "bytemuck error: {}", bytemuck.to_string()),
            Self::OutOfBounds(expected,got) => write!(f, "out of bounds: boundary is {:#x}, got {:#x} instead", expected, got),
            Self::InvalidPointer(ptr) => write!(f, "invalid pointer: {:p}", ptr),
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
impl std::convert::From<PodCastError> for Error {
    fn from(bm_err: PodCastError) -> Self {
        Self::BytemuckError(bm_err)
    }
}
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

/// Convert the given reference of type ```T``` to a [`u8`](u8) [slice](slice).
///
/// `T` requires the trait [`Pod`](Pod) from the [bytemuck](bytemuck) library.
pub fn ref_to_bytes<T: Pod>(data: &T) -> Result<&[u8], Error> {
    if std::mem::size_of::<T>() == 0 { Ok(&[]) }
    else { let result = try_cast_slice::<T, u8>(core::slice::from_ref(data))?; Ok(result) }
}

/// Convert the given slice reference of type ```T``` to a [`u8`](u8) [slice](slice).
///
/// `T` requires the trait [`Pod`](Pod) from the [bytemuck](bytemuck) library.
pub fn slice_ref_to_bytes<T: Pod>(data: &[T]) -> Result<&[u8], Error> {
    let result = try_cast_slice::<T, u8>(data)?; Ok(result)
}

/// Convert the given reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
///
/// `T` requires the trait [`Pod`](Pod) from the [bytemuck](bytemuck) library.
pub fn ref_to_mut_bytes<T: Pod>(data: &mut T) -> Result<&mut [u8], Error> {
    if std::mem::size_of::<T>() == 0 { Ok(&mut []) }
    else { let result = try_cast_slice_mut::<T, u8>(core::slice::from_mut(data))?; Ok(result) }
}

/// Convert the given slice reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
///
/// `T` requires the trait [`Pod`](Pod) from the [bytemuck](bytemuck) library.
pub fn slice_ref_to_mut_bytes<T: Pod>(data: &mut [T]) -> Result<&mut [u8], Error> {
    let result = try_cast_slice_mut::<T, u8>(data)?; Ok(result)
}

/// The trait by which all buffer objects are derived.
pub trait Buffer {
    /// Get the length of this `Buffer` object.
    fn len(&self) -> usize;
    /// Get the `Buffer` object as a pointer.
    fn as_ptr(&self) -> *const u8;
    /// Get the `Buffer` object as a mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut u8;
    /// Get the `Buffer` object as a slice.
    fn as_slice(&self) -> &[u8];
    /// Get the `Buffer` object as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [u8];

    /// Get a pointer to the end of the buffer.
    ///
    /// Note that this pointer is not safe to use because it points at the very end of
    /// the buffer, which contains no data. It is merely a reference pointer for calculations
    /// such as boundaries and size.
    fn eob(&self) -> *const u8 {
        unsafe { self.as_ptr().add(self.len()) }
    }
    /// Get a pointer range of this buffer. See [slice::as_ptr_range](slice::as_ptr_range) for more details.
    fn as_ptr_range(&self) -> std::ops::Range<*const u8> {
        std::ops::Range::<*const u8> { start: self.as_ptr(), end: self.eob() }
    }
    /// Get a mutable pointer range of this buffer. See [slice::as_mut_ptr_range](slice::as_mut_ptr_range) for more details.
    fn as_mut_ptr_range(&mut self) -> std::ops::Range<*mut u8> {
        std::ops::Range::<*mut u8> { start: self.as_mut_ptr(), end: self.eob() as *mut u8 }
    }
    /// Check whether or not this buffer is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Validate that the given *pointer* object is within the range of this buffer.
    fn validate_ptr(&self, ptr: *const u8) -> bool {
        self.as_ptr_range().contains(&ptr)
    }
    /// Convert an *offset* to a [`u8`](u8) pointer.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset is out of bounds
    /// of the buffer.
    fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        if offset >= self.len() {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_ptr().add(offset)) }
    }
    /// Convert an *offset* to a mutable [`u8`](u8) pointer.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset is out of bounds
    /// of the buffer.
    fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        if offset >= self.len() {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_mut_ptr().add(offset)) }
    }
    /// Convert a *pointer* to an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the given pointer is not
    /// within the range of this buffer.
    fn ptr_to_offset(&self, ptr: *const u8) -> Result<usize, Error> {
        if !self.validate_ptr(ptr) { return Err(Error::InvalidPointer(ptr)); }

        Ok(ptr as usize - self.as_ptr() as usize)
    }
    /// Convert a given reference to an object into an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) if this reference did not come from
    /// this buffer.
    fn ref_to_offset<T>(&self, data: &T) -> Result<usize, Error> {
        let ptr = data as *const T as *const u8;

        self.ptr_to_offset(ptr)
    }
    /// Convert a given [slice](slice) reference to an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the slice reference
    /// did not originate from this buffer.
    fn slice_ref_to_offset<T>(&self, data: &[T]) -> Result<usize, Error> {
        let ptr = data.as_ptr() as *const u8;

        self.ptr_to_offset(ptr)
    }
    /// Convert this buffer to a [`u8`](u8) [`Vec`](Vec) object.
    fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
    /// Swap two bytes at the given offsets. This panics if the offsets are out of bounds. See [`slice::swap`](slice::swap)
    /// for more details.
    fn swap(&mut self, a: usize, b: usize) {
        self.as_mut_slice().swap(a, b);
    }
    /// Reverse the buffer. See [`slice::reverse`](slice::reverse) for more details.
    fn reverse(&mut self) {
        self.as_mut_slice().reverse();
    }
    /// Return an iterator object ([`BufferIter`](BufferIter)) into the buffer.
    fn iter(&self) -> BufferIter<'_> {
        BufferIter { buffer: self.as_slice(), index: 0 }
    }
    /// Return a mutable iterator object ([`BufferIterMut`](BufferIterMut)) into the buffer.
    fn iter_mut(&mut self) -> BufferIterMut<'_> {
        BufferIterMut { buffer: self.as_mut_slice(), index: 0 }
    }
    /// Save this buffer to disk.
    fn save<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Error> {
        std::fs::write(filename, self.as_slice())?;
        Ok(())
    }
    /// Get the given byte or range of bytes from the buffer. See [`slice::get`](slice::get) for more details.
    fn get<I: std::slice::SliceIndex<[u8]>>(&self, index: I) -> Option<&I::Output> {
        self.as_slice().get(index)
    }
    /// Get the given byte or range of bytes from the buffer as mutable. See [`slice::get_mut`](slice::get_mut) for more details.
    fn get_mut<I: std::slice::SliceIndex<[u8]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.as_mut_slice().get_mut(index)
    }
    /// Get a reference to a given object within the buffer. Typically the main interface by which objects are retrieved.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the object's size plus
    /// the offset results in an out-of-bounds event. `T` is required to be of the [`Pod`](Pod) trait from [bytemuck](bytemuck).
    ///
    /// # Example
    /// ```rust
    /// use hex;
    /// use pkbuffer::{Buffer, VecBuffer};
    ///
    /// let buffer = VecBuffer::from_data(&hex::decode("facebabedeadbeef").unwrap());
    ///
    /// let dword = buffer.get_ref::<u32>(4);
    /// assert!(dword.is_ok());
    /// assert_eq!(*dword.unwrap(), 0xEFBEADDE);
    /// ```
    fn get_ref<T: Pod>(&self, offset: usize) -> Result<&T, Error> {
        let size = std::mem::size_of::<T>();

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        let bytes = self.get_slice_ref::<u8>(offset, size)?;
        let result = try_from_bytes::<T>(bytes)?;
        Ok(result)
    }
    /// Get a mutable reference to a given object within the buffer. See [`Buffer::get_ref`](Buffer::get_ref).
    fn get_mut_ref<T: Pod>(&mut self, offset: usize) -> Result<&mut T, Error> {
        let size = std::mem::size_of::<T>();

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        let bytes = self.get_mut_slice_ref::<u8>(offset, size)?;
        let result = try_from_bytes_mut::<T>(bytes)?;
        Ok(result)
    }
    /// Convert a given reference to a mutable reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    fn make_mut_ref<T: Pod>(&mut self, data: &T) -> Result<&mut T, Error> {
        let offset = self.ref_to_offset(data)?;
        self.get_mut_ref::<T>(offset)
    }
    /// Gets a slice reference of type *T* at the given *offset* with the given *size*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or
    /// the offset plus its size goes out of bounds of the buffer.
    ///
    /// # Example
    /// ```rust
    /// use hex;
    /// use pkbuffer::{Buffer, VecBuffer};
    ///
    /// let buffer = VecBuffer::from_data(&hex::decode("f00dbeef1deadead").unwrap());
    ///
    /// let slice = buffer.get_slice_ref::<u16>(0, 4);
    /// assert!(slice.is_ok());
    /// assert_eq!(slice.unwrap(), [0x0DF0, 0xEFBE, 0xEA1D, 0xADDE]);
    /// ```
    fn get_slice_ref<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        let ptr = self.offset_to_ptr(offset)?;
        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+real_size));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const T, size)) }
    }
    /// Gets a mutable slice reference of type *T* at the given *offset* with the given *size*.
    /// See [`Buffer::get_slice_ref`](Buffer::get_slice_ref).
    fn get_mut_slice_ref<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        let ptr = self.offset_to_mut_ptr(offset)?;
        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+real_size));
        }

        unsafe { Ok(std::slice::from_raw_parts_mut(ptr as *mut T, size)) }
    }
    /// Convert a given [slice](slice) reference to a mutable [slice](slice) reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    fn make_mut_slice_ref<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        let offset = self.ptr_to_offset(data.as_ptr() as *const u8)?;
        self.get_mut_slice_ref::<T>(offset, data.len())
    }
    /// Read an arbitrary *size* amount of bytes from the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the read runs out of boundaries.
    fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        self.get_slice_ref::<u8>(offset, size)
    }
    /// Read an arbitrary *size* amount of bytes from the given *offset*, but mutable.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the read runs out of boundaries.
    fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        self.get_mut_slice_ref::<u8>(offset, size)
    }
    /// Write an arbitrary [`u8`](u8) [slice](slice) to the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries
    /// of the buffer.
    fn write<B: AsRef<[u8]>>(&mut self, offset: usize, data: B) -> Result<(), Error> {
        let buf = data.as_ref();
        let from_ptr = buf.as_ptr();
        let to_ptr = self.offset_to_mut_ptr(offset)?;
        let size = buf.len();

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        unsafe { std::ptr::copy(from_ptr, to_ptr, size); }

        Ok(())
    }
    /// Write a given object of type *T* to the given buffer at the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn write_ref<T: Pod>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.write(offset, bytes)
    }
    /// Write a given slice object of type *T* to the given buffer at the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn write_slice_ref<T: Pod>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.write(offset, bytes)
    }
    /// Start the buffer object with the given byte data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn start_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        self.write(0, data)
    }
    /// Start the buffer with the given reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn start_with_ref<T: Pod>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.start_with(bytes)
    }
    /// Start the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn start_with_slice_ref<T: Pod>(&mut self, data: &[T]) -> Result<(), Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.start_with(bytes)
    }
    /// End the buffer object with the given byte data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn end_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        let buf = data.as_ref();

        if buf.len() > self.len() { return Err(Error::OutOfBounds(self.len(),buf.len())); }
        
        self.write(self.len()-buf.len(), data)
    }
    /// End the buffer with the given reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn end_with_ref<T: Pod>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.end_with(bytes)
    }
    /// End the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn end_with_slice_ref<T: Pod>(&mut self, data: &[T]) -> Result<(), Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.end_with(bytes)
    }
    /// Search for the given [`u8`](u8) [slice](slice) *data* within the given buffer.
    ///
    /// On success, this returns an iterator to all found offsets which match the given search term.
    /// Typically, the error returned is an [`Error::OutOfBounds`](Error::OutOfBounds) error, when the search
    /// term exceeds the size of the buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use hex;
    /// use pkbuffer::{Buffer, VecBuffer};
    ///
    /// let buffer = VecBuffer::from_data(&hex::decode("beefbeefb33fbeefbeef").unwrap());
    /// let search = buffer.search(&[0xBE, 0xEF]);
    /// assert!(search.is_ok());
    ///
    /// let mut results = search.unwrap();
    /// assert_eq!(results.next().unwrap(), 0);
    /// assert_eq!(results.next().unwrap(), 2);
    /// assert_eq!(results.next().unwrap(), 6);
    /// assert_eq!(results.next().unwrap(), 8);
    /// assert!(results.next().is_none());
    ///
    /// // alternatively, you can snatch up the search results into a Vec
    /// let search_results = buffer.search(&[0xBE, 0xEF]).unwrap().collect::<Vec<usize>>();
    /// assert_eq!(search_results, [0,2,6,8]);
    /// ```
    fn search<'a, B: AsRef<[u8]>>(&'a self, data: B) -> Result<BufferSearchIter<'a>, Error> {
        BufferSearchIter::new(self.as_slice(), data)
    }
    /// Search for the following reference of type *T*. This converts the object into a [`u8`](u8) [slice](slice).
    /// See [`Buffer::search`](Buffer::search).
    fn search_ref<'a, T: Pod>(&'a self, data: &T) -> Result<BufferSearchIter<'a>, Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.search(bytes)
    }
    /// Search for the following slice reference of type *T*. This converts the slice into a [`u8`](u8) [slice](slice).
    /// See [`Buffer::search`](Buffer::search).
    fn search_slice_ref<'a, T: Pod>(&'a self, data: &[T]) -> Result<BufferSearchIter<'a>, Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.search(bytes)
    }
    /// Check if this buffer contains the following [`u8`](u8) [slice](slice) sequence.
    fn contains<B: AsRef<[u8]>>(&self, data: B) -> bool {
        let buf = data.as_ref();

        if buf.len() > self.len() { return false; }

        let mut offset = 0usize;

        for i in 0..self.len() {
            if offset >= buf.len() { break; }

            if *self.get(i).unwrap() != buf[offset] { offset = 0; continue; }
            else { offset += 1; }
        }

        offset == buf.len()
    }
    /// Check if this buffer contains the following object of type *T*.
    fn contains_ref<T: Pod>(&self, data: &T) -> Result<bool, Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        Ok(self.contains(bytes))
    }
    /// Check if this buffer contains the following slice of type *T*.
    fn contains_slice_ref<T: Pod>(&self, data: &[T]) -> Result<bool, Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        Ok(self.contains(bytes))
    }
    /// Check if this buffer starts with the byte sequence *needle*. See [`slice::starts_with`](slice::starts_with).
    fn starts_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().starts_with(needle.as_ref())
    }
    /// Check if this buffer ends with the byte sequence *needle*. See [`slice::ends_with`](slice::ends_with).
    fn ends_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().ends_with(needle.as_ref())
    }
    /// Rotate the buffer left at midpoint *mid*. See [`slice::rotate_left`](slice::rotate_left).
    fn rotate_left(&mut self, mid: usize) {
        self.as_mut_slice().rotate_left(mid);
    }
    /// Rotate the buffer right at midpoint *mid*. See [`slice::rotate_right`](slice::rotate_right).
    fn rotate_right(&mut self, mid: usize) {
        self.as_mut_slice().rotate_right(mid);
    }
    /// Fill the given buffer with the given *value*. See [`slice::fill`](slice::fill).
    fn fill(&mut self, value: u8) {
        self.as_mut_slice().fill(value);
    }
    /// Fill the given buffer with the given closure *f*. See [`slice::fill_with`](slice::fill_with).
    fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> u8
    {
        self.as_mut_slice().fill_with(f)
    }
    /// Clone the given [`u8`](u8) [slice](slice) data *src* into the given buffer.
    fn clone_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().clone_from_slice(src.as_ref());
    }
    /// Copy the given [`u8`](u8) [slice](slice) data *src* into the given buffer.
    fn copy_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().copy_from_slice(src.as_ref());
    }
    /// Copy from within the given buffer. See [`slice::copy_within`](slice::copy_within).
    fn copy_within<R>(&mut self, src: R, dest: usize)
    where
        R: std::ops::RangeBounds<usize>
    {
        self.as_mut_slice().copy_within(src, dest)
    }
    /// Swap the data in this buffer with the given [`u8`](u8) [slice](slice) reference.
    fn swap_with_data<B: AsMut<[u8]>>(&mut self, mut other: B) {
        self.as_mut_slice().swap_with_slice(other.as_mut());
    }
    /// Check if this buffer is ASCII. See [`slice::is_ascii`](slice::is_ascii).
    fn is_ascii(&self) -> bool {
        self.as_slice().is_ascii()
    }
    /// Check if this buffer is equal while ignoring case of letters. See [`slice::eq_ignore_ascii_case`](slice::eq_ignore_ascii_case).
    fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.as_slice().eq_ignore_ascii_case(other)
    }
    /// Make this buffer ASCII uppercase. See [`slice::make_ascii_uppercase`](slice::make_ascii_uppercase).
    fn make_ascii_uppercase(&mut self) {
        self.as_mut_slice().make_ascii_uppercase();
    }
    /// Make this buffer ASCII lowercase. See [`slice::make_ascii_lowercase`](slice::make_ascii_lowercase).
    fn make_ascii_lowercase(&mut self) {
        self.as_mut_slice().make_ascii_lowercase();
    }
    /// Sort this buffer. See [`slice::sort`](slice::sort).
    fn sort(&mut self) {
        self.as_mut_slice().sort();
    }
    /// Sort by the given closure comparing each individual byte. See [`slice::sort_by`](slice::sort_by).
    fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&u8, &u8) -> std::cmp::Ordering
    {
        self.as_mut_slice().sort_by(compare);
    }
    /// Sorts the slice with a key extraction function. See [`slice::sort_by_key`](slice::sort_by_key).
    fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> K,
        K: std::cmp::Ord,
    {
        self.as_mut_slice().sort_by_key(f);
    }
    /// Creates a new `Buffer` object by repeating the current buffer *n* times. See [`slice::repeat`](slice::repeat).
    fn repeat(&self, n: usize) -> Vec<u8> {
        self.as_slice().repeat(n)
    }
}

/// An iterator for a [`Buffer`](Buffer) object.
pub struct BufferIter<'a> {
    buffer: &'a [u8],
    index: usize,
}
impl<'a> BufferIter<'a> {
    /// Creates a new [`Buffer`](Buffer) iterator object.
    pub fn new(buffer: &'a [u8], index: usize) -> Self {
        Self { buffer, index }
    }
}
impl<'a> Iterator for BufferIter<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<&'a u8> {
        if self.index >= self.buffer.len() { return None; }

        let result = &self.buffer[self.index];
        self.index += 1;

        Some(result)
    }
}

/// A mutable iterator for a [`Buffer`](Buffer) object.
pub struct BufferIterMut<'a> {
    buffer: &'a mut [u8],
    index: usize,
}
impl<'a> BufferIterMut<'a> {
    /// Create a new mutable iterator for a [`Buffer`](Buffer) object.
    pub fn new(buffer: &'a mut [u8], index: usize) -> Self {
        Self { buffer, index }
    }
}
impl<'a> Iterator for BufferIterMut<'a> {
    type Item = &'a mut u8;

    fn next(&mut self) -> Option<&'a mut u8> {
        if self.index >= self.buffer.len() { return None; }

        let ptr = &mut self.buffer[self.index] as *mut u8;
        let result = unsafe { &mut *ptr };
        self.index += 1;

        Some(result)
    }
}

/// An iterator for searching over a [`Buffer`](Buffer)'s space for a given binary search term.
pub struct BufferSearchIter<'a> {
    buffer: &'a [u8],
    term: Vec<u8>,
    offsets: Vec<usize>,
    offset_index: usize,
}
impl<'a> BufferSearchIter<'a> {
    /// Create a new search iterator over a buffer reference. Typically you'll just want to call [`Buffer::search`](Buffer::search) instead,
    /// but this essentially does the same thing.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the search term is longer than the buffer.
    pub fn new<B: AsRef<[u8]>>(buffer: &'a [u8], term: B) -> Result<Self, Error> {
        let search = term.as_ref();

        if search.len() > buffer.len() { return Err(Error::OutOfBounds(buffer.len(),search.len())); }
        
        let mut offsets = Vec::<usize>::new();

        for i in 0..=(buffer.len() - search.len()) {
            if buffer[i] == search[0] { offsets.push(i); }
        }

        Ok(Self { buffer: buffer, term: search.to_vec(), offsets: offsets, offset_index: 0 })
    }
}
impl<'a> Iterator for BufferSearchIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.offset_index >= self.offsets.len() { return None; }

            let offset = self.offsets[self.offset_index];
            self.offset_index += 1;

            let found_slice = &self.buffer[offset..offset+self.term.len()];
            if found_slice == self.term.as_slice() { return Some(offset); }
        }
    }
}

/// A [`Buffer`](Buffer) object backed by a pointer/size pair. Use this buffer type
/// when accessing unowned memory or arbitrary allocated memory.
#[derive(Copy, Clone, Eq, Debug)]
pub struct PtrBuffer {
    pointer: *const u8,
    size: usize,
}
impl PtrBuffer {
    /// Create a new buffer object with a given *pointer* and *size*. Just make sure the pointer outlives
    /// the object and not the other way around.
    pub fn new(pointer: *const u8, size: usize) -> Self {
        Self { pointer, size }
    }
    /// Set the new pointer of this buffer.
    pub fn set_pointer(&mut self, pointer: *const u8) {
        self.pointer = pointer;
    }
    /// Set the new size of this buffer.
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    /// Create a new `PtrBuffer` object within the bounds of the current buffer.
    pub fn sub_buffer(&self, offset: usize, size: usize) -> Result<Self, Error> {
        if offset >= self.len() {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        unsafe { Ok(Self::new(self.as_ptr().add(offset), size)) }
    }
    /// Split this buffer into two separate buffers at the given splitpoint *mid*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if this split goes out of bounds of the buffer.
    pub fn split_at(&self, mid: usize) -> Result<(Self, Self), Error> {
        if mid > self.len() { return Err(Error::OutOfBounds(self.len(),mid)); }
        
        Ok((Self::new(self.as_ptr(),mid),
            Self::new(unsafe { self.as_ptr().add(mid) }, self.len() - mid)))
    }
}
impl Buffer for PtrBuffer {
    /// Get the length of this `PtrBuffer` object.
    fn len(&self) -> usize {
        self.size
    }
    /// Get the `PtrBuffer` object as a pointer.
    fn as_ptr(&self) -> *const u8 {
        self.pointer
    }
    /// Get the `PtrBuffer` object as a mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.pointer as *mut u8
    }
    /// Get the `PtrBuffer` object as a slice.
    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.size) }
    }
    /// Get the `PtrBuffer` object as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size) }
    }
}
impl PartialEq<[u8]> for PtrBuffer {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}
impl<const N: usize> PartialEq<[u8; N]> for PtrBuffer {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.as_slice() == other
    }
}
impl PartialEq<Vec<u8>> for PtrBuffer {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: Buffer> PartialEq<T> for PtrBuffer {
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::Index<Idx> for PtrBuffer {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        self.as_slice().index(index)
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::IndexMut<Idx> for PtrBuffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}
impl std::convert::AsRef<[u8]> for PtrBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl std::convert::AsMut<[u8]> for PtrBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
impl std::hash::Hash for PtrBuffer {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher
    {
        self.as_slice().hash(state);
    }
    fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: std::hash::Hasher
    {
        data.iter().for_each(|x| x.hash(state));
    }
}
impl std::iter::IntoIterator for PtrBuffer {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

/// An owned-data [`Buffer`](Buffer) object.
#[derive(Clone, Eq, Debug)]
pub struct VecBuffer {
    data: Vec<u8>,
}
impl VecBuffer {
    /// Create a new ```VecBuffer``` object, similar to [`Vec::new`](Vec::new).
    pub fn new() -> Self {
        Self { data: Vec::<u8>::new() }
    }
    /// Create a new `VecBuffer` object with initialization data.
    pub fn from_data<B: AsRef<[u8]>>(data: B) -> Self {
        Self { data: data.as_ref().to_vec() }
    }
    /// Create a new ```VecBuffer``` from the given file data.
    pub fn from_file<P: AsRef<std::path::Path>>(filename: P) -> Result<Self, Error> {
        let data = match std::fs::read(filename) {
            Ok(d) => d,
            Err(e) => return Err(Error::from(e)),
        };

        Ok(Self { data })
    }
    /// Create a new ```VecBuffer``` with a given starting size. This will zero out the
    /// buffer on initialization.
    pub fn with_initial_size(size: usize) -> Self {
        Self::from_data(&vec![0u8; size])
    }
    /// Create a [`PtrBuffer`](PtrBuffer) object from this `VecBuffer` object.
    pub fn as_ptr_buffer(&self) -> PtrBuffer {
        PtrBuffer::new(self.data.as_ptr(), self.data.len())
    }
    /// Appends the given data to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append<B: AsRef<[u8]>>(&mut self, data: B) {
        self.data.append(&mut data.as_ref().to_vec());
    }
    /// Appends the given reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_ref<T: Pod>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.append(bytes); Ok(())
    }
    /// Appends the given slice reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_slice_ref<T: Pod>(&mut self, data: &[T]) -> Result<(), Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.append(bytes); Ok(())
    }
    /// Insert a given *element* at the given *offset*, expanding the vector by one. See [`Vec::insert`](Vec::insert).
    pub fn insert(&mut self, offset: usize, element: u8) {
        self.data.insert(offset, element);
    }
    /// Remove a given element at the given *offset*, shrinking the vector by one. See [`Vec::remove`](Vec::remove).
    pub fn remove(&mut self, offset: usize) {
        self.data.remove(offset);
    }
    /// Retains only the elements specified by the predicate. See [`Vec::retain`](Vec::retain).
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> bool
    {
        self.data.retain(f);
    }
    /// Push a byte onto the end of the buffer. See [`Vec::push`](Vec::push).
    pub fn push(&mut self, v: u8) {
        self.data.push(v);
    }
    /// Pop a byte from the end of the buffer. See [`Vec::pop`](Vec::pop).
    pub fn pop(&mut self) -> Option<u8> {
        self.data.pop()
    }
    /// Clear the given buffer.
    pub fn clear(&mut self) {
        self.data.clear();
    }
    /// Split off into another ```VecBuffer``` instance at the given midpoint. See [`Vec::split_off`](Vec::split_off).
    pub fn split_off(&mut self, at: usize) -> Self {
        let data = self.data.split_off(at);
        Self::from_data(&data)
    }
    /// Resize the buffer to *new size*, filling with the given closure *f*. See [`Vec::resize_with`](Vec::resize_with).
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> u8
    {
        self.data.resize_with(new_len, f);
    }
    /// Resize the given buffer and fill the void with the given *value*. See [`Vec::resize`](Vec::resize).
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.data.resize(new_len, value);
    }
    /// Truncate the size of the buffer to the given *len*.
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
    }
    /// Deduplicate the values in this buffer. See [`Vec::dedup`](Vec::dedup).
    pub fn dedup(&mut self) {
        self.data.dedup();
    }
}
impl Buffer for VecBuffer {
    /// Get the length of this `VecBuffer` object.
    fn len(&self) -> usize {
        self.data.len()
    }
    /// Get the `VecBuffer` object as a pointer.
    fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    /// Get the `VecBuffer` object as a mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    /// Get the `VecBuffer` object as a slice.
    fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
    /// Get the `VecBuffer` object as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [u8]
    {
        self.data.as_mut_slice()
    }
}
impl PartialEq<[u8]> for VecBuffer {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}
impl<const N: usize> PartialEq<[u8; N]> for VecBuffer {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.as_slice() == other
    }
}
impl PartialEq<Vec<u8>> for VecBuffer {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: Buffer> PartialEq<T> for VecBuffer {
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::Index<Idx> for VecBuffer {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        self.data.index(index)
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::IndexMut<Idx> for VecBuffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}
impl std::convert::AsRef<[u8]> for VecBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl std::convert::AsMut<[u8]> for VecBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
impl std::hash::Hash for VecBuffer {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher
    {
        self.data.hash(state);
    }
    fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: std::hash::Hasher
    {
        data.iter().for_each(|x| x.hash(state));
    }
}
impl std::iter::IntoIterator for VecBuffer {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}
