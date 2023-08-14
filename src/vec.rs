use crate::{ref_to_bytes, slice_ref_to_bytes, Buffer, Castable, Error, PtrBuffer};

/// An owned-data [`Buffer`](Buffer) object.
#[derive(Clone, Eq, Debug)]
pub struct VecBuffer {
    data: Vec<u8>,
}
impl VecBuffer {
    /// Create a new ```VecBuffer``` object, similar to [`Vec::new`](Vec::new).
    pub fn new() -> Self {
        Self {
            data: Vec::<u8>::new(),
        }
    }
    /// Create a new `VecBuffer` object with initialization data.
    pub fn from_data<B: AsRef<[u8]>>(data: B) -> Self {
        Self {
            data: data.as_ref().to_vec(),
        }
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
        self.data.extend_from_slice(data.as_ref());
    }
    /// Appends the given reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_ref<T: Castable>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
    /// Appends the given slice reference to the end of the buffer. This resizes and expands the underlying vector.
    pub fn append_slice_ref<T: Castable>(&mut self, data: &[T]) -> Result<(), Error> {
        let bytes = slice_ref_to_bytes::<T>(data)?;
        self.append(bytes);
        Ok(())
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
        F: FnMut(&u8) -> bool,
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
        F: FnMut() -> u8,
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
    fn as_mut_slice(&mut self) -> &mut [u8] {
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
        H: std::hash::Hasher,
    {
        self.data.hash(state);
    }
    fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: std::hash::Hasher,
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
