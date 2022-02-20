//! [PKBuffer](https://github.com/frank2/exe-rs) is a library built for arbitrary casting of data structures
//! onto segments of memory! This includes sections of unowned memory, such as examining the headers of a
//! currently running executable. It creates an interface for reading and writing data structures to an
//! arbitrary buffer of bytes.
//!
//! For example:
//! ```rust
//! use pkbuffer::VecBuffer;
//!
//! #[repr(packed)]
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
//! The buffer comes in two forms: *pointer form* ([`Buffer`](Buffer)) and
//! *allocated form* ([`VecBuffer`](VecBuffer)). Each of these structures come
//! in handy for different reasons. [`Buffer`](Buffer)'s extra implementations
//! are based on the [slice](slice) object, whereas [`VecBuffer`](VecBuffer)'s
//! extra implementations are based on the [Vec](Vec) object.

#[cfg(test)]
mod tests;

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
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(io) => write!(f, "i/o error: {}", io.to_string()),
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

/// Convert the given reference of type ```T``` to a [`u8`](u8) [slice](slice).
pub fn ref_to_bytes<T>(data: &T) -> &[u8] {
    let ptr = data as *const T as *const u8;
    let size = std::mem::size_of::<T>();

    unsafe { std::slice::from_raw_parts(ptr, size) }
}

/// Convert the given slice reference of type ```T``` to a [`u8`](u8) [slice](slice).
pub fn slice_ref_to_bytes<T>(data: &[T]) -> &[u8] {
    let ptr = data.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * data.len();

    unsafe { std::slice::from_raw_parts(ptr, size) }
}

/// Convert the given reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
pub fn ref_to_mut_bytes<T>(data: &mut T) -> &mut [u8] {
    let ptr = data as *mut T as *mut u8;
    let size = std::mem::size_of::<T>();

    unsafe { std::slice::from_raw_parts_mut(ptr, size) }
}

/// Convert the given slice reference of type ```T``` to a mutable [`u8`](u8) [slice](slice).
pub fn slice_ref_to_mut_bytes<T>(data: &mut [T]) -> &mut [u8] {
    let ptr = data.as_mut_ptr() as *mut u8;
    let size = std::mem::size_of::<T>() * data.len();

    unsafe { std::slice::from_raw_parts_mut(ptr, size) }
}

/// The core buffer object which handles the majority of buffer-like interactions.
///
/// The only difference between the internals of a ```Buffer``` structure and a [`Vec`](Vec) object is
/// the ```Buffer``` object does not maintain a running capacity, merely a pointer and a size. As a
/// result, ```Buffer``` objects are more like [slice](slice) objects. So much of its functionality
/// is derived from them, and its documentation will be referring you to that page often.
///
/// Usually you will want to operate on an owned data object of some kind, not memory you don't control.
/// In that case, you should probably look at [VecBuffer](VecBuffer) instead.
#[derive(Copy, Clone, Eq, Debug)]
pub struct Buffer {
    pointer: *const u8,
    size: usize,
}
impl Buffer {
    /// Create a new buffer object with a given *pointer* and *size*. Just make sure the pointer outlives
    /// the object and not the other way around.
    pub fn new(pointer: *const u8, size: usize) -> Self {
        Self { pointer, size }
    }
    /// Create a new buffer from a [`u8`](u8) [slice](slice) reference.
    ///
    /// This function converts the slice to a pointer and takes its length as arguments for construction.
    pub fn from_ref<B: AsRef<[u8]>>(data: B) -> Self {
        let buf = data.as_ref();
        
        Self::new(buf.as_ptr(), buf.len())
    }
    /// Set the new pointer of this buffer.
    pub fn set_pointer(&mut self, pointer: *const u8) {
        self.pointer = pointer;
    }
    /// Set the new size of this buffer.
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }
    /// Create a new ```Buffer``` object within the bounds of the current buffer.
    pub fn sub_buffer(&self, offset: usize, size: usize) -> Result<Self, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.size,offset));
        }

        if offset+size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+size));
        }

        unsafe { Ok(Self::new(self.pointer.add(offset), size)) }
    }
    /// Get the length of this buffer.
    pub fn len(&self) -> usize {
        self.size
    }
    /// Check whether or not this buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get the base pointer of this buffer.
    pub fn as_ptr(&self) -> *const u8 {
        self.pointer
    }
    /// Get a mutable base pointer of this buffer.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.pointer as *mut u8
    }
    /// Get a pointer to the end of the buffer.
    ///
    /// Note that this pointer is not safe to use because it points at the very end of
    /// the buffer, which contains no data. It is merely a reference pointer for calculations
    /// such as boundaries and size.
    pub fn eob(&self) -> *const u8 {
        unsafe { self.as_ptr().add(self.len()) }
    }
    /// Get a pointer range of this buffer. See [slice::as_ptr_range](slice::as_ptr_range) for more details.
    pub fn as_ptr_range(&self) -> std::ops::Range<*const u8> {
        std::ops::Range::<*const u8> { start: self.as_ptr(), end: self.eob() }
    }
    /// Get a mutable pointer range of this buffer. See [slice::as_mut_ptr_range](slice::as_mut_ptr_range) for more details.
    pub fn as_mut_ptr_range(&self) -> std::ops::Range<*mut u8> {
        std::ops::Range::<*mut u8> { start: self.as_mut_ptr(), end: self.eob() as *mut u8 }
    }
    /// Represent this buffer as a [`u8`](u8) [slice](slice) object.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.size) }
    }
    /// Represent this buffer as a mutable [`u8`](u8) [slice](slice) object.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size) }
    }
    /// Validate that the given *pointer* object is within the range of this buffer.
    pub fn validate_ptr(&self, ptr: *const u8) -> bool {
        self.as_ptr_range().contains(&ptr)
    }
    /// Convert an *offset* to a [`u8`](u8) pointer.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset is out of bounds
    /// of the buffer.
    pub fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_ptr().add(offset)) }
    }
    /// Convert an *offset* to a mutable [`u8`](u8) pointer.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset is out of bounds
    /// of the buffer.
    pub fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_mut_ptr().add(offset)) }
    }
    /// Convert a *pointer* to an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the given pointer is not
    /// within the range of this buffer.
    pub fn ptr_to_offset(&self, ptr: *const u8) -> Result<usize, Error> {
        if !self.validate_ptr(ptr) { return Err(Error::InvalidPointer(ptr)); }

        Ok(ptr as usize - self.as_ptr() as usize)
    }
    /// Convert a given reference to an object into an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) if this reference did not come from
    /// this buffer.
    pub fn ref_to_offset<T>(&self, data: &T) -> Result<usize, Error> {
        let ptr = data as *const T as *const u8;

        self.ptr_to_offset(ptr)
    }
    /// Convert a given [slice](slice) reference to an offset into the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the slice reference
    /// did not originate from this buffer.
    pub fn slice_ref_to_offset<T>(&self, data: &[T]) -> Result<usize, Error> {
        let ptr = data.as_ptr() as *const u8;

        self.ptr_to_offset(ptr)
    }
    /// Convert this buffer to a [`u8`](u8) [`Vec`](Vec) object.
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
    /// Swap two bytes at the given offsets. This panics if the offsets are out of bounds. See [`slice::swap`](slice::swap)
    /// for more details.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.as_mut_slice().swap(a, b);
    }
    /// Reverse the buffer. See [`slice::reverse`](slice::reverse) for more details.
    pub fn reverse(&mut self) {
        self.as_mut_slice().reverse();
    }
    /// Return an iterator object ([`BufferIter`](BufferIter)) into the buffer.
    pub fn iter(&self) -> BufferIter<'_> {
        BufferIter { buffer: self, index: 0 }
    }
    /// Return a mutable iterator object ([`BufferIterMut`](BufferIterMut)) into the buffer.
    pub fn iter_mut(&mut self) -> BufferIterMut<'_> {
        BufferIterMut { buffer: self, index: 0 }
    }
    /// Save this buffer to disk.
    pub fn save<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Error> {
        match std::fs::write(filename, self.as_slice()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::IoError(e)),
        }
    }
    /// Get the given byte or range of bytes from the buffer. See [`slice::get`](slice::get) for more details.
    pub fn get<I: std::slice::SliceIndex<[u8]>>(&self, index: I) -> Option<&I::Output> {
        self.as_slice().get(index)
    }
    /// Get the given byte or range of bytes from the buffer as mutable. See [`slice::get_mut`](slice::get_mut) for more details.
    pub fn get_mut<I: std::slice::SliceIndex<[u8]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.as_mut_slice().get_mut(index)
    }
    /// Get a reference to a given object within the buffer. Typically the main interface by which objects are retrieved.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the object's size plus
    /// the offset results in an out-of-bounds event.
    ///
    /// # Example
    /// ```rust
    /// use hex;
    /// use pkbuffer::Buffer;
    ///
    /// let data = hex::decode("facebabedeadbeef").unwrap();
    /// let buffer = Buffer::from_ref(&data);
    ///
    /// let dword = buffer.get_ref::<u32>(4);
    /// assert!(dword.is_ok());
    /// assert_eq!(*dword.unwrap(), 0xEFBEADDE);
    /// ```
    pub fn get_ref<T>(&self, offset: usize) -> Result<&T, Error> {
        let ptr = match self.offset_to_ptr(offset) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let size = std::mem::size_of::<T>();

        if offset+size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+size));
        }

        unsafe { Ok(&*(ptr as *const T)) }
    }
    /// Get a mutable reference to a given object within the buffer. See [`Buffer::get_ref`](Buffer::get_ref).
    pub fn get_mut_ref<T>(&mut self, offset: usize) -> Result<&mut T, Error> {
        let ptr = match self.offset_to_mut_ptr(offset) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let size = std::mem::size_of::<T>();

        if offset+size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+size));
        }

        unsafe { Ok(&mut *(ptr as *const T as *mut T)) }
    }
    /// Convert a given reference to a mutable reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    pub fn make_mut_ref<T>(&mut self, data: &T) -> Result<&mut T, Error> {
        let offset = match self.ref_to_offset(data) {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

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
    /// use pkbuffer::Buffer;
    ///
    /// let data = hex::decode("f00dbeef1deadead").unwrap();
    /// let buffer = Buffer::from_ref(&data);
    ///
    /// let slice = buffer.get_slice_ref::<u16>(0, 4);
    /// assert!(slice.is_ok());
    /// assert_eq!(slice.unwrap(), [0x0DF0, 0xEFBE, 0xEA1D, 0xADDE]);
    /// ```
    pub fn get_slice_ref<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        let ptr = match self.offset_to_ptr(offset) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+real_size));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const T, size)) }
    }
    /// Gets a mutable slice reference of type *T* at the given *offset* with the given *size*.
    /// See [`Buffer::get_slice_ref`](Buffer::get_slice_ref).
    pub fn get_mut_slice_ref<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        let ptr = match self.offset_to_mut_ptr(offset) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+real_size));
        }

        unsafe { Ok(std::slice::from_raw_parts_mut(ptr as *mut T, size)) }
    }
    /// Convert a given [slice](slice) reference to a mutable [slice](slice) reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    pub fn make_mut_slice_ref<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        let offset = match self.ptr_to_offset(data.as_ptr() as *const u8) {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        self.get_mut_slice_ref::<T>(offset, data.len())
    }
    /// Read an arbitrary *size* amount of bytes from the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the read runs out of boundaries.
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        self.get_slice_ref::<u8>(offset, size)
    }
    /// Read an arbitrary *size* amount of bytes from the given *offset*, but mutable.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the read runs out of boundaries.
    pub fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        self.get_mut_slice_ref::<u8>(offset, size)
    }
    /// Write an arbitrary [`u8`](u8) [slice](slice) to the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries
    /// of the buffer.
    pub fn write<B: AsRef<[u8]>>(&mut self, offset: usize, data: B) -> Result<(), Error> {
        let buf = data.as_ref();
        let from_ptr = buf.as_ptr();
        let to_ptr = match self.offset_to_mut_ptr(offset) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let size = buf.len();

        if offset+size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+size));
        }

        unsafe { std::ptr::copy(from_ptr, to_ptr, size); }

        Ok(())
    }
    /// Write a given object of type *T* to the given buffer at the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn write_ref<T>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        self.write(offset, ref_to_bytes::<T>(data))
    }
    /// Write a given slice object of type *T* to the given buffer at the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn write_slice_ref<T>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        self.write(offset, slice_ref_to_bytes::<T>(data))
    }
    /// Start the buffer object with the given byte data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn start_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        self.write(0, data)
    }
    /// Start the buffer with the given reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn start_with_ref<T>(&mut self, data: &T) -> Result<(), Error> {
        self.start_with(ref_to_bytes::<T>(data))
    }
    /// Start the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn start_with_slice_ref<T>(&mut self, data: &[T]) -> Result<(), Error> {
        self.start_with(slice_ref_to_bytes::<T>(data))
    }
    /// End the buffer object with the given byte data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn end_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        let buf = data.as_ref();

        if buf.len() > self.len() { return Err(Error::OutOfBounds(self.len(),buf.len())); }
        
        self.write(self.len()-buf.len(), data)
    }
    /// End the buffer with the given reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn end_with_ref<T>(&mut self, data: &T) -> Result<(), Error> {
        self.end_with(ref_to_bytes::<T>(data))
    }
    /// End the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    pub fn end_with_slice_ref<T>(&mut self, data: &[T]) -> Result<(), Error> {
        self.end_with(slice_ref_to_bytes::<T>(data))
    }
    /// Search for the given [`u8`](u8) [slice](slice) *data* within the given buffer.
    ///
    /// On success, this returns an iterator to all found offsets which match the given search term.
    /// Typically, the error returned is an [`Error::OutOfBounds`](Error::OutOfBounds) error, when the search
    /// term exceeds the size of the buffer.
    pub fn search<'a, B: AsRef<[u8]>>(&'a self, data: B) -> Result<BufferSearchIter<'a>, Error> {
        BufferSearchIter::new(self, data)
    }
    /// Search for the following reference of type *T*. This converts the object into a [`u8`](u8) [slice](slice).
    /// See [`Buffer::search`](Buffer::search).
    pub fn search_ref<'a, T>(&'a self, data: &T) -> Result<BufferSearchIter<'a>, Error> {
        self.search(ref_to_bytes::<T>(data))
    }
    /// Search for the following slice reference of type *T*. This converts the slice into a [`u8`](u8) [slice](slice).
    /// See [`Buffer::search`](Buffer::search).
    pub fn search_slice_ref<'a, T>(&'a self, data: &[T]) -> Result<BufferSearchIter<'a>, Error> {
        self.search(slice_ref_to_bytes::<T>(data))
    }
    /// Check if this buffer contains the following [`u8`](u8) [slice](slice) sequence.
    pub fn contains<B: AsRef<[u8]>>(&self, data: B) -> bool {
        let buf = data.as_ref();

        if buf.len() > self.len() { return false; }

        let mut offset = 0usize;

        for i in 0..self.len() {
            if offset >= buf.len() { break; }

            if self[i] != buf[offset] { offset = 0; continue; }
            else { offset += 1; }
        }

        offset == buf.len()
    }
    /// Check if this buffer contains the following object of type *T*.
    pub fn contains_ref<T>(&self, data: &T) -> bool {
        self.contains(ref_to_bytes::<T>(data))
    }
    /// Check if this buffer contains the following slice of type *T*.
    pub fn contains_slice_ref<T>(&self, data: &[T]) -> bool {
        self.contains(slice_ref_to_bytes::<T>(data))
    }
    /// Check if this buffer starts with the byte sequence *needle*. See [`slice::starts_with`](slice::starts_with).
    pub fn starts_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().starts_with(needle.as_ref())
    }
    /// Check if this buffer ends with the byte sequence *needle*. See [`slice::ends_with`](slice::ends_with).
    pub fn ends_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().ends_with(needle.as_ref())
    }
    /// Rotate the buffer left at midpoint *mid*. See [`slice::rotate_left`](slice::rotate_left).
    pub fn rotate_left(&mut self, mid: usize) {
        self.as_mut_slice().rotate_left(mid);
    }
    /// Rotate the buffer right at midpoint *mid*. See [`slice::rotate_right`](slice::rotate_right).
    pub fn rotate_right(&mut self, mid: usize) {
        self.as_mut_slice().rotate_right(mid);
    }
    /// Fill the given buffer with the given *value*. See [`slice::fill`](slice::fill).
    pub fn fill(&mut self, value: u8) {
        self.as_mut_slice().fill(value);
    }
    /// Fill the given buffer with the given closure *f*. See [`slice::fill_with`](slice::fill_with).
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> u8
    {
        self.as_mut_slice().fill_with(f)
    }
    /// Clone the given [`u8`](u8) [slice](slice) data *src* into the given buffer.
    pub fn clone_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().clone_from_slice(src.as_ref());
    }
    /// Copy the given [`u8`](u8) [slice](slice) data *src* into the given buffer.
    pub fn copy_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().copy_from_slice(src.as_ref());
    }
    /// Copy from within the given buffer. See [`slice::copy_within`](slice::copy_within).
    pub fn copy_within<R>(&mut self, src: R, dest: usize)
    where
        R: std::ops::RangeBounds<usize>
    {
        self.as_mut_slice().copy_within(src, dest)
    }
    /// Swap the data in this buffer with the given [`u8`](u8) [slice](slice) reference.
    pub fn swap_with_data<B: AsMut<[u8]>>(&mut self, mut other: B) {
        self.as_mut_slice().swap_with_slice(other.as_mut());
    }
    /// Check if this buffer is ASCII. See [`slice::is_ascii`](slice::is_ascii).
    pub fn is_ascii(&self) -> bool {
        self.as_slice().is_ascii()
    }
    /// Check if this buffer is equal while ignoring case of letters. See [`slice::eq_ignore_ascii_case`](slice::eq_ignore_ascii_case).
    pub fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.as_slice().eq_ignore_ascii_case(other)
    }
    /// Make this buffer ASCII uppercase. See [`slice::make_ascii_uppercase`](slice::make_ascii_uppercase).
    pub fn make_ascii_uppercase(&mut self) {
        self.as_mut_slice().make_ascii_uppercase();
    }
    /// Make this buffer ASCII lowercase. See [`slice::make_ascii_lowercase`](slice::make_ascii_lowercase).
    pub fn make_ascii_lowercase(&mut self) {
        self.as_mut_slice().make_ascii_lowercase();
    }
    /// Sort this buffer. See [`slice::sort`](slice::sort).
    pub fn sort(&mut self) {
        self.as_mut_slice().sort();
    }
    /// Sort by the given closure comparing each individual byte. See [`slice::sort_by`](slice::sort_by).
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&u8, &u8) -> std::cmp::Ordering
    {
        self.as_mut_slice().sort_by(compare);
    }
    /// Sorts the slice with a key extraction function. See [`slice::sort_by_key`](slice::sort_by_key).
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> K,
        K: std::cmp::Ord,
    {
        self.as_mut_slice().sort_by_key(f);
    }
    /// Creates a new vector by repeating the current buffer *n* times. See [`slice::repeat`](slice::repeat).
    pub fn repeat(&self, n: usize) -> Vec<u8> {
        self.as_slice().repeat(n)
    }
    /// Split this buffer into two separate buffers at the given splitpoint *mid*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if this split goes out of bounds of the buffer.
    pub fn split_at(&self, mid: usize) -> Result<(Buffer, Buffer), Error> {
        if mid > self.len() { return Err(Error::OutOfBounds(self.len(),mid)); }
        
        Ok((Buffer::new(self.as_ptr(),mid),
            Buffer::new(unsafe { self.as_ptr().add(mid) }, self.len() - mid)))
    }
}
impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl PartialEq<[u8]> for Buffer {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}
impl<const N: usize> PartialEq<[u8; N]> for Buffer {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.as_slice() == other
    }
}
impl PartialEq<Vec<u8>> for Buffer {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl PartialEq<VecBuffer> for Buffer {
    fn eq(&self, other: &VecBuffer) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::Index<Idx> for Buffer {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        self.as_slice().index(index)
    }
}
impl<Idx: std::slice::SliceIndex<[u8]>> std::ops::IndexMut<Idx> for Buffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}
impl std::convert::AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl std::convert::AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
impl std::hash::Hash for Buffer {
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
impl<'a> std::iter::IntoIterator for &'a Buffer {
    type Item = &'a u8;
    type IntoIter = BufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator for a [`Buffer`](Buffer) object.
pub struct BufferIter<'a> {
    buffer: &'a Buffer,
    index: usize,
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
    buffer: &'a mut Buffer,
    index: usize,
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

pub struct BufferSearchIter<'a> {
    buffer: &'a Buffer,
    term: Vec<u8>,
    offsets: Vec<usize>,
    offset_index: usize,
}
impl<'a> BufferSearchIter<'a> {
    pub fn new<B: AsRef<[u8]>>(buffer: &'a Buffer, term: B) -> Result<Self, Error> {
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

            let found_slice = match self.buffer.read(offset, self.term.len()) {
                Ok(s) => s,
                Err(_) => return None,
            };

            if found_slice == self.term.as_slice() { return Some(offset); }
        }
    }
}

/// An owned-data [`Buffer`](Buffer) object.
///
/// Since pointers are so scary, this is the primary vessel by which most people will be accessing
/// the buffer interface. (Plus the fact that the [`Buffer`](Buffer) object can defy mutability rules in Rust
/// if you're not careful!) Though because it's built on top of the [`Buffer`](Buffer) object, documentation
/// will refer to it frequently.
#[derive(Eq, Debug)]
pub struct VecBuffer {
    data: Vec<u8>,
    buffer: Buffer,
}
impl VecBuffer {
    /// Create a new ```VecBuffer``` object, similar to [`Vec::new`](Vec::new).
    pub fn new() -> Self {
        let data = Vec::<u8>::new();
        let buffer = Buffer::from_ref(&data);

        Self { data, buffer }
    }
    /// Clone the data from *data* and create a new ```VecBuffer``` object.
    ///
    /// Unlike [`Buffer::from_ref`](Buffer::from_ref), this function instead clones the data
    /// given and stores it in a vector within the struct. It intializes a [`Buffer`](Buffer) object
    /// from this cloned data.
    pub fn from_data<B: AsRef<[u8]>>(data: B) -> Self {
        let vec = data.as_ref().to_vec();
        let buffer = Buffer::from_ref(&vec);

        Self { data: vec, buffer: buffer }
    }
    /// Create a new ```VecBuffer``` from the given file data.
    pub fn from_file<P: AsRef<std::path::Path>>(filename: P) -> Result<Self, Error> {
        let data = match std::fs::read(filename) {
            Ok(d) => d,
            Err(e) => return Err(Error::IoError(e)),
        };

        let buffer = Buffer::from_ref(&data);

        Ok(Self { data, buffer })
    }
    /// Create a new ```VecBuffer``` with a given starting size. This will zero out the
    /// buffer on initialization.
    pub fn with_initial_size(size: usize) -> Self {
        Self::from_data(&vec![0u8; size])
    }
    fn reassign(&mut self) {
        self.buffer = Buffer::from_ref(&self.data);
    }
    /// Get a sub-buffer from this buffer. See [`Buffer::sub_buffer`](Buffer::sub_buffer).
    pub fn sub_buffer(&self, offset: usize, size: usize) -> Result<Buffer, Error> {
        self.buffer.sub_buffer(offset, size)
    }
    /// Get the length of this buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Convert the buffer into a pure [`u8`](u8) [`Vec`](Vec) object.
    pub fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }
    /// Convert the buffer into a [`u8`](u8) [slice](slice) object.
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
    /// Convert the buffer into a mutable [`u8`](u8) [slice](slice) object.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }
    /// Convert the buffer into a [`u8`](u8) pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    /// Convert the buffer into a mutable [`u8`](u8) pointer.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    /// Get the buffer as a [`Buffer`](Buffer) object.
    pub fn as_buffer(&self) -> &Buffer {
        &self.buffer
    }
    /// Get the buffer as a mutable [`Buffer`](Buffer) object.
    pub fn as_mut_buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
    /// See [`Buffer::validate_ptr`](Buffer::validate_ptr).
    pub fn validate_ptr(&self, ptr: *const u8) -> bool {
        self.buffer.validate_ptr(ptr)
    }
    /// See [`Buffer::offset_to_ptr`](Buffer::offset_to_ptr).
    pub fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        self.buffer.offset_to_ptr(offset)
    }
    /// See [`Buffer::offset_to_mut_ptr`](Buffer::offset_to_mut_ptr).
    pub fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        self.buffer.offset_to_mut_ptr(offset)
    }
    /// See [`Buffer::ptr_to_offset`](Buffer::ptr_to_offset).
    pub fn ptr_to_offset(&self, ptr: *const u8) -> Result<usize, Error> {
        self.buffer.ptr_to_offset(ptr)
    }
    /// See [`Buffer::ref_to_offset`](Buffer::ref_to_offset).
    pub fn ref_to_offset<T>(&self, data: &T) -> Result<usize, Error> {
        self.buffer.ref_to_offset::<T>(data)
    }
    /// See [`Buffer::slice_ref_to_offset`](Buffer::slice_ref_to_offset).
    pub fn slice_ref_to_offset<T>(&self, data: &[T]) -> Result<usize, Error> {
        self.buffer.slice_ref_to_offset::<T>(data)
    }
    /// See [`Buffer::save`](Buffer::save).
    pub fn save<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Error> {
        self.buffer.save(filename)
    }
    /// See [`Buffer::get`](Buffer::get).
    pub fn get<I: std::slice::SliceIndex<[u8]>>(&self, index: I) -> Option<&I::Output> {
        self.data.get(index)
    }
    /// See [`Buffer::get_mut`](Buffer::get_mut).
    pub fn get_mut<I: std::slice::SliceIndex<[u8]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.data.get_mut(index)
    }
    /// See [`Buffer::get_ref`](Buffer::get_ref).
    pub fn get_ref<T>(&self, offset: usize) -> Result<&T, Error> {
        self.buffer.get_ref::<T>(offset)
    }
    /// See [`Buffer::get_mut_ref`](Buffer::get_mut_ref).
    pub fn get_mut_ref<T>(&mut self, offset: usize) -> Result<&mut T, Error> {
        self.buffer.get_mut_ref::<T>(offset)
    }
    /// See [`Buffer::make_mut_ref`](Buffer::make_mut_ref).
    pub fn make_mut_ref<T>(&mut self, data: &T) -> Result<&mut T, Error> {
        self.buffer.make_mut_ref::<T>(data)
    }
    /// See [`Buffer::get_slice_ref`](Buffer::get_slice_ref).
    pub fn get_slice_ref<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        self.buffer.get_slice_ref::<T>(offset, size)
    }
    /// See [`Buffer::get_mut_slice_ref`](Buffer::get_mut_slice_ref).
    pub fn get_mut_slice_ref<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        self.buffer.get_mut_slice_ref::<T>(offset, size)
    }
    /// See [`Buffer::make_mut_slice_ref`](Buffer::make_mut_slice_ref).
    pub fn make_mut_slice_ref<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        self.buffer.make_mut_slice_ref::<T>(data)
    }
    /// See [`Buffer::read`](Buffer::read).
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        self.buffer.read(offset, size)
    }
    /// See [`Buffer::read_mut`](Buffer::read_mut).
    pub fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        self.buffer.read_mut(offset, size)
    }
    /// See [`Buffer::write`](Buffer::write).
    pub fn write<B: AsRef<[u8]>>(&mut self, offset: usize, data: B) -> Result<(), Error> {
        self.buffer.write(offset, data)
    }
    /// See [`Buffer::write_ref`](Buffer::write_ref).
    pub fn write_ref<T>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        self.buffer.write_ref::<T>(offset, data)
    }
    /// See [`Buffer::write_slice_ref`](Buffer::write_slice_ref).
    pub fn write_slice_ref<T>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        self.buffer.write_slice_ref::<T>(offset, data)
    }
    /// See [`Buffer::start_with`](Buffer::start_with).
    pub fn start_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        self.buffer.start_with(data)
    }
    /// See [`Buffer::start_with_ref`](Buffer::start_with_ref).
    pub fn start_with_ref<T>(&mut self, data: &T) -> Result<(), Error> {
        self.buffer.start_with_ref::<T>(data)
    }
    /// See [`Buffer::start_with_slice_ref`](Buffer::start_with_slice_ref).
    pub fn start_with_slice_ref<T>(&mut self, data: &[T]) -> Result<(), Error> {
        self.buffer.start_with_slice_ref::<T>(data)
    }
    /// See [`Buffer::end_with`](Buffer::end_with).
    pub fn end_with<B: AsRef<[u8]>>(&mut self, data: B) -> Result<(), Error> {
        self.buffer.end_with(data)
    }
    /// See [`Buffer::end_with_ref`](Buffer::end_with_ref).
    pub fn end_with_ref<T>(&mut self, data: &T) -> Result<(), Error> {
        self.buffer.end_with_ref::<T>(data)
    }
    /// See [`Buffer::end_with_slice_ref`](Buffer::end_with_slice_ref).
    pub fn end_with_slice_ref<T>(&mut self, data: &[T]) -> Result<(), Error> {
        self.buffer.end_with_slice_ref::<T>(data)
    }
    /// Appends the given data to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append<B: AsRef<[u8]>>(&mut self, data: B) {
        self.data.append(&mut data.as_ref().to_vec());
        self.reassign();
    }
    /// Appends the given reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_ref<T>(&mut self, data: &T) {
        self.append(ref_to_bytes::<T>(data));
    }
    /// Appends the given slice reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_slice_ref<T>(&mut self, data: &[T]) {
        self.append(slice_ref_to_bytes::<T>(data));
    }
    /// See [`Buffer::search`](Buffer::search).
    pub fn search<'a, B: AsRef<[u8]>>(&'a self, data: B) -> Result<BufferSearchIter<'a>, Error> {
        self.buffer.search(data)
    }
    /// See [`Buffer::search_ref`](Buffer::search_ref).
    pub fn search_ref<'a, T>(&'a self, data: &T) -> Result<BufferSearchIter<'a>, Error> {
        self.buffer.search_ref::<T>(data)
    }
    /// See [`Buffer::search_slice_ref`](Buffer::search_slice_ref).
    pub fn search_slice_ref<'a, T>(&'a self, data: &[T]) -> Result<BufferSearchIter<'a>, Error> {
        self.buffer.search_slice_ref::<T>(data)
    }
    /// See [`Buffer::contains`](Buffer::contains).
    pub fn contains<B: AsRef<[u8]>>(&self, data: B) -> bool {
        self.buffer.contains(data)
    }
    /// See [`Buffer::contains_ref`](Buffer::contains_ref).
    pub fn contains_ref<T>(&self, data: &T) -> bool {
        self.buffer.contains_ref::<T>(data)
    }
    /// See [`Buffer::contains_slice_ref`](Buffer::contains_slice_ref).
    pub fn contains_slice_ref<T>(&self, data: &[T]) -> bool {
        self.buffer.contains_slice_ref::<T>(data)
    }
    /// See [`Buffer::starts_with`](Buffer::starts_with).
    pub fn starts_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.buffer.starts_with(needle)
    }
    /// See [`Buffer::ends_with`](Buffer::ends_with).
    pub fn ends_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.buffer.ends_with(needle)
    }
    /// Insert a given *element* at the given *offset*, expanding the vector by one. See [`Vec::insert`](Vec::insert).
    pub fn insert(&mut self, offset: usize, element: u8) {
        self.data.insert(offset, element);
        self.reassign();
    }
    /// Remove a given element at the given *offset*, shrinking the vector by one. See [`Vec::remove`](Vec::remove).
    pub fn remove(&mut self, offset: usize) {
        self.data.remove(offset);
        self.reassign();
    }
    /// Retains only the elements specified by the predicate. See [`Vec::retain`](Vec::retain).
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> bool
    {
        self.data.retain(f);
        self.reassign();
    }
    /// Push a byte onto the end of the buffer. See [`Vec::push`](Vec::push).
    pub fn push(&mut self, v: u8) {
        self.data.push(v);
        self.reassign();
    }
    /// Pop a byte from the end of the buffer. See [`Vec::pop`](Vec::pop).
    pub fn pop(&mut self) -> Option<u8> {
        let result = self.data.pop();
        if result.is_some() { self.reassign(); }
        result
    }
    /// Clear the given buffer.
    pub fn clear(&mut self) {
        self.data.clear();
        self.reassign();
    }
    /// Split off into another ```VecBuffer``` instance at the given midpoint. See [`Vec::split_off`](Vec::split_off).
    pub fn split_off(&mut self, at: usize) -> Self {
        let data = self.data.split_off(at);
        self.reassign();
        Self::from_data(&data)
    }
    /// Resize the buffer to *new size*, filling with the given closure *f*. See [`Vec::resize_with`](Vec::resize_with).
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> u8
    {
        self.data.resize_with(new_len, f);
        self.reassign();
    }
    /// Resize the given buffer and fill the void with the given *value*. See [`Vec::resize`](Vec::resize).
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.data.resize(new_len, value);
        self.reassign();
    }
    /// Truncate the size of the buffer to the given *len*.
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
        self.reassign();
    }
    /// Extend this buffer from the given [`u8`](u8) [slice](slice).
    pub fn extend_from_data<B: AsRef<[u8]>>(&mut self, other: B) {
        self.data.extend_from_slice(other.as_ref());
        self.reassign();
    }
    /// Deduplicate the values in this buffer. See [`Vec::dedup`](Vec::dedup).
    pub fn dedup(&mut self) {
        self.data.dedup();
        self.reassign();
    }
    /// Swap the two bytes at the given offsets. See [`slice::swap`](slice::swap).
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a,b);
    }
    /// Reverse the buffer. See [`slice::reverse`](slice::reverse).
    pub fn reverse(&mut self) {
        self.data.reverse();
    }
    /// Creates a [`BufferIter`](BufferIter) object, iterating over the values in the buffer.
    pub fn iter(&self) -> BufferIter<'_> {
        self.buffer.iter()
    }
    /// Creates a mutable [`BufferIterMut`](BufferIterMut) object, iterating over the values in the buffer.
    pub fn iter_mut(&mut self) -> BufferIterMut<'_> {
        self.buffer.iter_mut()
    }
    /// Splits the buffer into two [`Buffer`](Buffer) objects at the given midpoint.
    pub fn split_at(&self, mid: usize) -> Result<(Buffer, Buffer), Error> {
        self.buffer.split_at(mid)
    }
    /// Rotates the buffer left at the given midpoint, see [`slice::rotate_left`](slice::rotate_left).
    pub fn rotate_left(&mut self, mid: usize) {
        self.data.rotate_left(mid);
    }
    /// Rotates the buffer right at the given midpoint, see [`slice::rotate_right`](slice::rotate_right).
    pub fn rotate_right(&mut self, mid: usize) {
        self.data.rotate_right(mid);
    }
    /// Fill the buffer with the given value. See [`slice::fill`](slice::fill).
    pub fn fill(&mut self, value: u8) {
        self.data.fill(value);
    }
    /// Fill the buffer with the given closure *f*. See [`slice::fill_with`](slice::fill_with).
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> u8
    {
        self.data.fill_with(f);
    }
    /// Clone into the vector from the given [`u8`](u8) [slice](slice) reference.
    pub fn clone_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.data.clone_from_slice(src.as_ref());
    }
    /// Copy into the vector from the given [`u8`](u8) [slice](slice) reference.
    pub fn copy_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.data.copy_from_slice(src.as_ref());
    }
    /// Copy from within the given buffer. See [`slice::copy_within`](slice::copy_within).
    pub fn copy_within<R>(&mut self, src: R, dest: usize)
    where
        R: std::ops::RangeBounds<usize>
    {
        self.data.copy_within(src, dest);
    }
    /// Swap this buffer's data with the given [`u8`](u8) [slice](slice)'s data.
    pub fn swap_with_data<B: AsMut<[u8]>>(&mut self, mut other: B) {
        self.data.swap_with_slice(other.as_mut());
    }
    /// Check if the buffer is ASCII. See [`slice::is_ascii`](slice::is_ascii).
    pub fn is_ascii(&self) -> bool {
        self.data.is_ascii()
    }
    /// Check if this buffer is equal to the given [`u8`](u8) [slice](slice) reference, ignoring ASCII case.
    /// See [`slice::eq_ignore_ascii_case`](slice::eq_ignore_ascii_case).
    pub fn eq_ignore_ascii_case<B: AsRef<[u8]>>(&self, other: B) -> bool {
        self.data.eq_ignore_ascii_case(other.as_ref())
    }
    /// Make this buffer ASCII uppercase. See [`slice::make_ascii_uppercase`](slice::make_ascii_uppercase).
    pub fn make_ascii_uppercase(&mut self) {
        self.data.make_ascii_uppercase();
    }
    /// Make this buffer ASCII lowercase. See [`slice::make_ascii_lowercase`](slice::make_ascii_lowercase).
    pub fn make_ascii_lowercase(&mut self) {
        self.data.make_ascii_lowercase();
    }
    /// Sort this buffer. See [`slice::sort`](slice::sort).
    pub fn sort(&mut self) {
        self.data.sort();
    }
    /// Sort this buffer by a comparison closure. See [`slice::sort_by`](slice::sort_by).
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&u8, &u8) -> std::cmp::Ordering
    {
        self.data.sort_by(compare);
    }
    /// Sort this buffer by a comparison key. See [`slice::sort_by_key`](slice::sort_by_key).
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> K,
        K: std::cmp::Ord,
    {
        self.data.sort_by_key(f);
    }
    /// Repeat this buffer *n* times, creating a new ```VecBuffer``` object with the repeated data.
    pub fn repeat(&self, n: usize) -> Self {
        let data = self.data.repeat(n);

        Self::from_data(&data)
    }
}
impl Clone for VecBuffer {
    fn clone(&self) -> Self {
        let data = self.data.clone();
        let buffer = Buffer::from_ref(&data);
        
        Self { data, buffer }
    }
    fn clone_from(&mut self, source: &Self) {
        self.data = source.to_vec();
        self.buffer = Buffer::from_ref(&self.data);
    }
}
impl PartialEq for VecBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
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
impl PartialEq<Buffer> for VecBuffer {
    fn eq(&self, other: &Buffer) -> bool {
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
impl<'a> std::iter::IntoIterator for &'a VecBuffer {
    type Item = &'a u8;
    type IntoIter = BufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
