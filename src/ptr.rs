use crate::{Buffer, Error};

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
