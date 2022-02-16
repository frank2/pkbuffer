#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    OutOfBounds(usize,usize),
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

pub fn ref_to_bytes<T>(data: &T) -> &[u8] {
    let ptr = data as *const T as *const u8;
    let size = std::mem::size_of::<T>();

    unsafe { std::slice::from_raw_parts(ptr, size) }
}

pub fn slice_ref_to_bytes<T>(data: &[T]) -> &[u8] {
    let ptr = data.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * data.len();

    unsafe { std::slice::from_raw_parts(ptr, size) }
}

pub fn ref_to_mut_bytes<T>(data: &mut T) -> &mut [u8] {
    let ptr = data as *mut T as *mut u8;
    let size = std::mem::size_of::<T>();

    unsafe { std::slice::from_raw_parts_mut(ptr, size) }
}

pub fn slice_ref_to_mut_bytes<T>(data: &mut [T]) -> &mut [u8] {
    let ptr = data.as_mut_ptr() as *mut u8;
    let size = std::mem::size_of::<T>() * data.len();

    unsafe { std::slice::from_raw_parts_mut(ptr, size) }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Buffer {
    pointer: *const u8,
    size: usize,
}
impl Buffer {
    pub fn new(pointer: *const u8, size: usize) -> Self {
        Self { pointer, size }
    }
    pub fn from_ref<B: AsRef<[u8]>>(data: B) -> Self {
        let buf = data.as_ref();
        
        Self::new(buf.as_ptr(), buf.len())
    }
    pub fn sub_buffer(&self, offset: usize, size: usize) -> Result<Self, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.size,offset));
        }

        if offset+size > self.size {
            return Err(Error::OutOfBounds(self.size,offset+size));
        }

        unsafe { Ok(Self::new(self.pointer.add(offset), size)) }
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.pointer
    }
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.pointer as *mut u8
    }
    pub fn eob(&self) -> *const u8 {
        unsafe { self.as_ptr().add(self.len()) }
    }
    pub fn as_ptr_range(&self) -> std::ops::Range<*const u8> {
        std::ops::Range::<*const u8> { start: self.as_ptr(), end: self.eob() }
    }
    pub fn as_mut_ptr_range(&self) -> std::ops::Range<*mut u8> {
        std::ops::Range::<*mut u8> { start: self.as_mut_ptr(), end: self.eob() as *mut u8 }
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.size) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size) }
    }
    pub fn validate_ptr(&self, ptr: *const u8) -> bool {
        self.as_ptr_range().contains(&ptr)
    }
    pub fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_ptr().add(offset)) }
    }
    pub fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        if offset >= self.size {
            return Err(Error::OutOfBounds(self.len(),offset));
        }

        unsafe { Ok(self.as_mut_ptr().add(offset)) }
    }
    pub fn ptr_to_offset(&self, ptr: *const u8) -> Result<usize, Error> {
        if !self.validate_ptr(ptr) { return Err(Error::InvalidPointer(ptr)); }

        Ok(ptr as usize - self.as_ptr() as usize)
    }
    pub fn ref_to_offset<T>(&self, data: &T) -> Result<usize, Error> {
        let ptr = data as *const T as *const u8;

        self.ptr_to_offset(ptr)
    }
    pub fn slice_ref_to_offset<T>(&self, data: &[T]) -> Result<usize, Error> {
        let ptr = data.as_ptr() as *const u8;

        self.ptr_to_offset(ptr)
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
    pub fn swap(&mut self, a: usize, b: usize) {
        self.as_mut_slice().swap(a, b);
    }
    pub fn reverse(&mut self) {
        self.as_mut_slice().reverse();
    }
    pub fn iter(&self) -> BufferIter<'_> {
        BufferIter { buffer: self, index: 0 }
    }
    pub fn iter_mut(&mut self) -> BufferIterMut<'_> {
        BufferIterMut { buffer: self, index: 0 }
    }
    pub fn save<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Error> {
        match std::fs::write(filename, self.as_slice()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::IoError(e)),
        }
    }
    pub fn get<I: std::slice::SliceIndex<[u8]>>(&self, index: I) -> Option<&I::Output> {
        self.as_slice().get(index)
    }
    pub fn get_mut<I: std::slice::SliceIndex<[u8]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.as_mut_slice().get_mut(index)
    }
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
    pub fn make_mut_ref<T>(&mut self, data: &T) -> Result<&mut T, Error> {
        let offset = match self.ref_to_offset(data) {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        self.get_mut_ref::<T>(offset)
    }
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
    pub fn make_mut_slice_ref<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        let offset = match self.ptr_to_offset(data.as_ptr() as *const u8) {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        self.get_mut_slice_ref::<T>(offset, data.len())
    }
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        self.get_slice_ref::<u8>(offset, size)
    }
    pub fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        self.get_mut_slice_ref::<u8>(offset, size)
    }
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
    pub fn write_ref<T>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        self.write(offset, ref_to_bytes::<T>(data))
    }
    pub fn write_slice_ref<T>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        self.write(offset, slice_ref_to_bytes::<T>(data))
    }
    pub fn search<B: AsRef<[u8]>>(&self, data: B) -> Result<Option<Vec<usize>>, Error> {
        let search = data.as_ref();

        if search.len() > self.len() { return Err(Error::OutOfBounds(self.len(),search.len())); }

        let buffer_data = self.as_slice();
        let mut offsets = Vec::<usize>::new();

        for i in 0..=(self.len() - search.len()) {
            if buffer_data[i] == search[0] { offsets.push(i); }
        }

        let mut results = Vec::<usize>::new();

        for offset in &offsets {
            let found_slice = match self.read(*offset, search.len()) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };

            if found_slice == search { results.push(*offset); }
        }
        
        if results.len() == 0 { Ok(None) }
        else { Ok(Some(results)) }
    }
    pub fn search_ref<T>(&self, data: &T) -> Result<Option<Vec<usize>>, Error> {
        self.search(ref_to_bytes::<T>(data))
    }
    pub fn search_slice_ref<T>(&self, data: &[T]) -> Result<Option<Vec<usize>>, Error> {
        self.search(slice_ref_to_bytes::<T>(data))
    }
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
    pub fn contains_ref<T>(&self, data: &T) -> bool {
        self.contains(ref_to_bytes::<T>(data))
    }
    pub fn contains_slice_ref<T>(&self, data: &[T]) -> bool {
        self.contains(slice_ref_to_bytes::<T>(data))
    }
    pub fn starts_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().starts_with(needle.as_ref())
    }
    pub fn ends_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.as_slice().ends_with(needle.as_ref())
    }
    pub fn rotate_left(&mut self, mid: usize) {
        self.as_mut_slice().rotate_left(mid);
    }
    pub fn rotate_right(&mut self, mid: usize) {
        self.as_mut_slice().rotate_right(mid);
    }
    pub fn fill(&mut self, value: u8) {
        self.as_mut_slice().fill(value);
    }
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> u8
    {
        self.as_mut_slice().fill_with(f)
    }
    pub fn clone_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().clone_from_slice(src.as_ref());
    }
    pub fn copy_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.as_mut_slice().copy_from_slice(src.as_ref());
    }
    pub fn copy_within<R>(&mut self, src: R, dest: usize)
    where
        R: std::ops::RangeBounds<usize>
    {
        self.as_mut_slice().copy_within(src, dest)
    }
    pub fn swap_with_data<B: AsMut<[u8]>>(&mut self, mut other: B) {
        self.as_mut_slice().swap_with_slice(other.as_mut());
    }
    pub fn is_ascii(&self) -> bool {
        self.as_slice().is_ascii()
    }
    pub fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.as_slice().eq_ignore_ascii_case(other)
    }
    pub fn make_ascii_uppercase(&mut self) {
        self.as_mut_slice().make_ascii_uppercase();
    }
    pub fn make_ascii_lowercase(&mut self) {
        self.as_mut_slice().make_ascii_lowercase();
    }
    pub fn sort(&mut self) {
        self.as_mut_slice().sort();
    }
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&u8, &u8) -> std::cmp::Ordering
    {
        self.as_mut_slice().sort_by(compare);
    }
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> K,
        K: std::cmp::Ord,
    {
        self.as_mut_slice().sort_by_key(f);
    }
    pub fn repeat(&self, n: usize) -> Vec<u8> {
        self.as_slice().repeat(n)
    }
    pub fn split_at(&self, mid: usize) -> Result<(Buffer, Buffer), Error> {
        if mid > self.len() { return Err(Error::OutOfBounds(self.len(),mid)); }
        
        Ok((Buffer::new(self.as_ptr(),mid),
            Buffer::new(unsafe { self.as_ptr().add(mid) }, self.len() - mid)))
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VecBuffer {
    data: Vec<u8>,
    buffer: Buffer,
}
impl VecBuffer {
    pub fn new() -> Self {
        let data = Vec::<u8>::new();
        let buffer = Buffer::from_ref(&data);

        Self { data, buffer }
    }
    pub fn from_data<B: AsRef<[u8]>>(data: B) -> Self {
        let vec = data.as_ref().to_vec();
        let buffer = Buffer::from_ref(&vec);

        Self { data: vec, buffer: buffer }
    }
    pub fn from_file<P: AsRef<std::path::Path>>(filename: P) -> Result<Self, Error> {
        let data = match std::fs::read(filename) {
            Ok(d) => d,
            Err(e) => return Err(Error::IoError(e)),
        };

        let buffer = Buffer::from_ref(&data);

        Ok(Self { data, buffer })
    }
    pub fn with_initial_size(size: usize) -> Self {
        Self::from_data(&vec![0u8; size])
    }
    fn reassign(&mut self) {
        self.buffer = Buffer::from_ref(&self.data);
    }
    pub fn sub_buffer(&self, offset: usize, size: usize) -> Result<Buffer, Error> {
        self.buffer.sub_buffer(offset, size)
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    pub fn as_buffer(&self) -> &Buffer {
        &self.buffer
    }
    pub fn as_mut_buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
    pub fn validate_ptr(&self, ptr: *const u8) -> bool {
        self.buffer.validate_ptr(ptr)
    }
    pub fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        self.buffer.offset_to_ptr(offset)
    }
    pub fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        self.buffer.offset_to_mut_ptr(offset)
    }
    pub fn ptr_to_offset(&self, ptr: *const u8) -> Result<usize, Error> {
        self.buffer.ptr_to_offset(ptr)
    }
    pub fn ref_to_offset<T>(&self, data: &T) -> Result<usize, Error> {
        self.buffer.ref_to_offset::<T>(data)
    }
    pub fn slice_ref_to_offset<T>(&self, data: &[T]) -> Result<usize, Error> {
        self.buffer.slice_ref_to_offset::<T>(data)
    }
    pub fn save<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Error> {
        self.buffer.save(filename)
    }
    pub fn get<I: std::slice::SliceIndex<[u8]>>(&self, index: I) -> Option<&I::Output> {
        self.data.get(index)
    }
    pub fn get_mut<I: std::slice::SliceIndex<[u8]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.data.get_mut(index)
    }
    pub fn get_ref<T>(&self, offset: usize) -> Result<&T, Error> {
        self.buffer.get_ref::<T>(offset)
    }
    pub fn get_mut_ref<T>(&mut self, offset: usize) -> Result<&mut T, Error> {
        self.buffer.get_mut_ref::<T>(offset)
    }
    pub fn make_mut_ref<T>(&mut self, data: &T) -> Result<&mut T, Error> {
        self.buffer.make_mut_ref::<T>(data)
    }
    pub fn get_slice_ref<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        self.buffer.get_slice_ref::<T>(offset, size)
    }
    pub fn get_mut_slice_ref<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        self.buffer.get_mut_slice_ref::<T>(offset, size)
    }
    pub fn make_mut_slice_ref<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        self.buffer.make_mut_slice_ref::<T>(data)
    }
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        self.buffer.read(offset, size)
    }
    pub fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        self.buffer.read_mut(offset, size)
    }
    pub fn write<B: AsRef<[u8]>>(&mut self, offset: usize, data: B) -> Result<(), Error> {
        self.buffer.write(offset, data)
    }
    pub fn write_ref<T>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        self.buffer.write_ref::<T>(offset, data)
    }
    pub fn write_slice_ref<T>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        self.buffer.write_slice_ref::<T>(offset, data)
    }
    pub fn search<B: AsRef<[u8]>>(&self, data: B) -> Result<Option<Vec<usize>>, Error> {
        self.buffer.search(data)
    }
    pub fn search_ref<T>(&self, data: &T) -> Result<Option<Vec<usize>>, Error> {
        self.buffer.search_ref::<T>(data)
    }
    pub fn search_slice_ref<T>(&self, data: &[T]) -> Result<Option<Vec<usize>>, Error> {
        self.buffer.search_slice_ref::<T>(data)
    }
    pub fn contains<B: AsRef<[u8]>>(&self, data: B) -> bool {
        self.buffer.contains(data)
    }
    pub fn contains_ref<T>(&self, data: &T) -> bool {
        self.buffer.contains_ref::<T>(data)
    }
    pub fn contains_slice_ref<T>(&self, data: &[T]) -> bool {
        self.buffer.contains_slice_ref::<T>(data)
    }
    pub fn starts_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.buffer.starts_with(needle)
    }
    pub fn ends_with<B: AsRef<[u8]>>(&self, needle: B) -> bool {
        self.buffer.ends_with(needle)
    }
    pub fn insert(&mut self, offset: usize, element: u8) {
        self.data.insert(offset, element);
        self.reassign();
    }
    pub fn remove(&mut self, offset: usize) {
        self.data.remove(offset);
        self.reassign();
    }
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> bool
    {
        self.data.retain(f);
        self.reassign();
    }
    pub fn push(&mut self, v: u8) {
        self.data.push(v);
        self.reassign();
    }
    pub fn pop(&mut self) -> Option<u8> {
        let result = self.data.pop();
        if result.is_some() { self.reassign(); }
        result
    }
    pub fn append(&mut self, other: &mut Vec<u8>) {
        self.data.append(other);
        self.reassign();
    }
    pub fn clear(&mut self) {
        self.data.clear();
        self.reassign();
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        let data = self.data.split_off(at);
        self.reassign();
        Self::from_data(&data)
    }
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> u8
    {
        self.data.resize_with(new_len, f);
        self.reassign();
    }
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.data.resize(new_len, value);
        self.reassign();
    }
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
        self.reassign();
    }
    pub fn extend_from_data<B: AsRef<[u8]>>(&mut self, other: B) {
        self.data.extend_from_slice(other.as_ref());
        self.reassign();
    }
    pub fn dedup(&mut self) {
        self.data.dedup();
        self.reassign();
    }
    pub fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a,b);
    }
    pub fn reverse(&mut self) {
        self.data.reverse();
    }
    pub fn iter(&self) -> BufferIter<'_> {
        self.buffer.iter()
    }
    pub fn iter_mut(&mut self) -> BufferIterMut<'_> {
        self.buffer.iter_mut()
    }
    pub fn split_at(&self, mid: usize) -> Result<(Buffer, Buffer), Error> {
        self.buffer.split_at(mid)
    }
    pub fn rotate_left(&mut self, mid: usize) {
        self.data.rotate_left(mid);
    }
    pub fn rotate_right(&mut self, mid: usize) {
        self.data.rotate_right(mid);
    }
    pub fn fill(&mut self, value: u8) {
        self.data.fill(value);
    }
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> u8
    {
        self.data.fill_with(f);
    }
    pub fn clone_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.data.clone_from_slice(src.as_ref());
    }
    pub fn copy_from_data<B: AsRef<[u8]>>(&mut self, src: B) {
        self.data.copy_from_slice(src.as_ref());
    }
    pub fn copy_within<R>(&mut self, src: R, dest: usize)
    where
        R: std::ops::RangeBounds<usize>
    {
        self.data.copy_within(src, dest);
    }
    pub fn swap_with_data<B: AsMut<[u8]>>(&mut self, mut other: B) {
        self.data.swap_with_slice(other.as_mut());
    }
    pub fn is_ascii(&self) -> bool {
        self.data.is_ascii()
    }
    pub fn eq_ignore_ascii_case<B: AsRef<[u8]>>(&self, other: B) -> bool {
        self.data.eq_ignore_ascii_case(other.as_ref())
    }
    pub fn make_ascii_uppercase(&mut self) {
        self.data.make_ascii_uppercase();
    }
    pub fn make_ascii_lowercase(&mut self) {
        self.data.make_ascii_lowercase();
    }
    pub fn sort(&mut self) {
        self.data.sort();
    }
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&u8, &u8) -> std::cmp::Ordering
    {
        self.data.sort_by(compare);
    }
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&u8) -> K,
        K: std::cmp::Ord,
    {
        self.data.sort_by_key(f);
    }
    pub fn repeat(&self, n: usize) -> Self {
        let data = self.data.repeat(n);

        Self::from_data(&data)
    }
    pub fn to_ascii_uppercase(&self) -> Self {
        let data = self.data.to_ascii_uppercase();

        Self::from_data(&data)
    }
    pub fn to_ascii_lowercase(&self) -> Self {
        let data = self.data.to_ascii_lowercase();

        Self::from_data(&data)
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
