use crate::{Castable, Error, ref_to_bytes, slice_ref_to_bytes, bytes_to_ref, bytes_to_mut_ref};

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
    /// the offset results in an out-of-bounds event. `T` is required to be of the [`Castable`](Castable) trait from [bytemuck](bytemuck).
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
    fn get_ref<T: Castable>(&self, offset: usize) -> Result<&T, Error> {
        let size = std::mem::size_of::<T>();
        let bytes = self.get_slice_ref::<u8>(offset, size)?;
        bytes_to_ref::<T>(bytes)
    }
    /// Get a reference to a given object within the buffer, but in an unaligned way.
    ///
    /// Because of the way this function acquires a new reference, the [`Castable`](Castable) trait is unnecessary.
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the object's size plus
    /// the offset results in an out-of-bounds event.
    ///
    /// # Safety
    /// This is an unsafe function because it gets a reference that is not aligned to a proper boundary, which
    /// can trigger undefined behavior on some processors. If you're unsure of the alignment situation on your
    /// target processor, or unsure of the alignment situation in your data, it's best to use
    /// [`Buffer::get_ref`](Buffer::get_ref) instead.
    unsafe fn get_ref_unaligned<T>(&self, offset: usize) -> Result<&T, Error> {
        let ptr = self.offset_to_ptr(offset)?;
        let size = std::mem::size_of::<T>();

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        Ok(&*(ptr as *const T))
    }
    /// Get a reference regardless of potential alignment issues.
    ///
    /// It is not recommended you use this function if you're unaware of the alignment
    /// situation of your processor or data. See
    /// [`Buffer::get_ref_unaligned`](Buffer::get_ref_unaligned) for more details.
    unsafe fn force_get_ref<T: Castable>(&self, offset: usize) -> Result<&T, Error> {
        match self.get_ref::<T>(offset) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { self.get_ref_unaligned::<T>(offset) }
                else { Err(err) }
            },
        }
    }
    /// Get a mutable reference to a given object within the buffer. See [`Buffer::get_ref`](Buffer::get_ref).
    fn get_mut_ref<T: Castable>(&mut self, offset: usize) -> Result<&mut T, Error> {
        let size = std::mem::size_of::<T>();
        let bytes = self.get_mut_slice_ref::<u8>(offset, size)?;
        bytes_to_mut_ref::<T>(bytes)
    }
    /// Get a mutable reference to a given object within the buffer, but in an unaligned way.
    ///
    /// Because of the way this function acquires a new reference, the [`Castable`](Castable) trait is unnecessary.
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the object's size plus
    /// the offset results in an out-of-bounds event.
    ///
    /// # Safety
    /// This is an unsafe function because it gets a reference that is not aligned to a proper boundary, which
    /// can trigger undefined behavior on some processors. If you're unsure of the alignment situation on your
    /// target processor, or unsure of the alignment situation in your data, it's best to use
    /// [`Buffer::get_ref`](Buffer::get_ref) instead.
    unsafe fn get_mut_ref_unaligned<T>(&mut self, offset: usize) -> Result<&mut T, Error> {
        let ptr = self.offset_to_mut_ptr(offset)?;
        let size = std::mem::size_of::<T>();

        if offset+size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+size));
        }

        Ok(&mut *(ptr as *const T as *mut T))
    }
    /// Get a mutable reference regardless of potential alignment issues.
    ///
    /// It is not recommended you use this function if you're unaware of the alignment
    /// situation of your processor or data. See
    /// [`Buffer::get_mut_ref_unaligned`](Buffer::get_mut_ref_unaligned) for more details.
    unsafe fn force_get_mut_ref<T: Castable>(&mut self, offset: usize) -> Result<&mut T, Error> {
        // I'm unsure why the borrow checker is annoyed at this code, attempting to go out
        // of scope of the returned error (or even explicitly dropping it) still doesn't let
        // me borrow again, so just do some pointer magic to make a new reference. if you
        // know why this is causing a borrow issue and how to fix it please file a ticket on GitHub.
        let second_ref = &mut *(self as *mut Self);
        
        match self.get_mut_ref::<T>(offset) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { second_ref.get_mut_ref_unaligned::<T>(offset) }
                else { Err(err) }
            },
        }
    }
    /// Convert a given reference to a mutable reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    fn make_mut_ref<T: Castable>(&mut self, data: &T) -> Result<&mut T, Error> {
        let offset = self.ref_to_offset(data)?;
        self.get_mut_ref::<T>(offset)
    }
    /// Convert a given reference to a mutable reference without alignment guarantees.
    ///
    /// You should not do this unless you know your alignment situation. See
    /// [`Buffer::get_re_unalignedf`](Buffer::get_ref_unaligned) for an explanation as to why.
    unsafe fn make_mut_ref_unaligned<T>(&mut self, data: &T) -> Result<&mut T, Error> {
        let offset = self.ref_to_offset(data)?;
        self.get_mut_ref_unaligned::<T>(offset)
    }
    /// Convert an object to a mutable reference regardless of potential alignment issues.
    ///
    /// You should not do this unless you know your alignment situation. See
    /// [`Buffer::get_ref_unaligned`](Buffer::get_ref_unaligned) for an explanation as to why.
    unsafe fn force_make_mut_ref<T: Castable>(&mut self, data: &T) -> Result<&mut T, Error> {
        // I'm unsure why the borrow checker is annoyed at this code, attempting to go out
        // of scope of the returned error (or even explicitly dropping it) still doesn't let
        // me borrow again, so just do some pointer magic to make a new reference. if you
        // know why this is causing a borrow issue and how to fix it please file a ticket on GitHub.
        let second_ref = &mut *(self as *mut Self);
        let offset = self.ref_to_offset(data)?;
        
        match self.get_mut_ref::<T>(offset) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { second_ref.get_mut_ref_unaligned::<T>(offset) }
                else { Err(err) }
            },
        }
    }
    /// Gets a slice reference of type *T* at the given *offset* with the given *size*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the
    /// offset plus its size goes out of bounds of the buffer and returns
    /// [`Error::BadAlignment`](Error::BadAlignment) if the acquired slice is not aligned
    /// on the alignment boundary required by type *T*.
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
    fn get_slice_ref<T: Castable>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        let ptr = self.offset_to_ptr(offset)?;
        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+real_size));
        }

        let alignment = std::mem::align_of::<T>();

        if (ptr as usize) % alignment != 0 {
            return Err(Error::BadAlignment(alignment, (ptr as usize) % alignment));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const T, size)) }
    }
    /// Gets a slice ref of type *T* at the given *offset* regardless of potential alignment
    /// issues.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the offset or the offset
    /// plus its size go out of bounds of the buffer.
    ///
    /// # Safety
    /// This is an unsafe function because it gets a slice reference that is not aligned to a proper boundary, which
    /// can trigger undefined behavior on some processors. If you're unsure of the alignment situation on your
    /// target processor, or unsure of the alignment situation in your data, it's best to use
    /// [`Buffer::get_slice_ref`](Buffer::get_slice_ref) instead.
    unsafe fn get_slice_ref_unaligned<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        let ptr = self.offset_to_ptr(offset)?;
        let type_size = std::mem::size_of::<T>();
        let slice_end = offset + (size * type_size);

        if slice_end > self.len() {
            return Err(Error::OutOfBounds(self.len(), slice_end));
        }

        Ok(std::slice::from_raw_parts(ptr as *const T, size))
    }
    /// Get a slice reference regardless of potential alignment issues.
    ///
    /// It is not recommended you use this function if you're unaware of the alignment
    /// situation of your processor or data. See [`Buffer::get_ref`](Buffer::get_ref)
    /// for more details.
    unsafe fn force_get_slice_ref<T: Castable>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        match self.get_slice_ref::<T>(offset, size) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { self.get_slice_ref_unaligned::<T>(offset, size) }
                else { Err(err) }
            },
        }
    }
    /// Gets a mutable slice reference of type *T* at the given *offset* with the given *size*.
    /// See [`Buffer::get_slice_ref`](Buffer::get_slice_ref).
    fn get_mut_slice_ref<T: Castable>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        let ptr = self.offset_to_mut_ptr(offset)?;
        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+real_size));
        }

        let alignment = std::mem::align_of::<T>();

        if (ptr as usize) % alignment != 0 {
            return Err(Error::BadAlignment(alignment, offset % alignment));
        }

        unsafe { Ok(std::slice::from_raw_parts_mut(ptr as *mut T, size)) }
    }
    /// Gets a mutable slice reference of type *T* at the given *offset* with the given *size*,
    /// but without alignment checking. See [`Buffer::get_slice_ref_unaligned`](Buffer::get_slice_ref_unaligned).
    unsafe fn get_mut_slice_ref_unaligned<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        let ptr = self.offset_to_mut_ptr(offset)?;
        let real_size = std::mem::size_of::<T>() * size;
                
        if offset+real_size > self.len() {
            return Err(Error::OutOfBounds(self.len(),offset+real_size));
        }

        Ok(std::slice::from_raw_parts_mut(ptr as *mut T, size))
    }
    /// Get a mutable slice reference regardless of potential alignment issues.
    ///
    /// It is not recommended you use this function if you're unaware of the alignment
    /// situation of your processor or data. See
    /// [`Buffer::get_mut_slice_ref_unaligned`](Buffer::get_mut_slice_ref_unaligned) for more details.
    unsafe fn force_get_mut_slice_ref<T: Castable>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        // I'm unsure why the borrow checker is annoyed at this code, attempting to go out
        // of scope of the returned error (or even explicitly dropping it) still doesn't let
        // me borrow again, so just do some pointer magic to make a new reference. if you
        // know why this is causing a borrow issue and how to fix it please file a ticket on GitHub.
        let second_ref = &mut *(self as *mut Self);

        match self.get_mut_slice_ref::<T>(offset, size) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { second_ref.get_mut_slice_ref_unaligned::<T>(offset, size) }
                else { Err(err) }
            },
        }
    }
    /// Convert a given [slice](slice) reference to a mutable [slice](slice) reference within the buffer.
    ///
    /// Returns an [`Error::InvalidPointer`](Error::InvalidPointer) error if the reference did not
    /// originate from this buffer.
    fn make_mut_slice_ref<T: Castable>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        let offset = self.ptr_to_offset(data.as_ptr() as *const u8)?;
        self.get_mut_slice_ref::<T>(offset, data.len())
    }
    /// Convert a given slice reference to a mutable slice reference without alignment guarantees.
    ///
    /// You should not do this unless you know your alignment situation. See
    /// [`Buffer::get_mut_slice_ref_unaligned`](Buffer::get_mut_slice_ref_unaligned) for an explanation as to why.
    unsafe fn make_mut_slice_ref_unaligned<T>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        let offset = self.slice_ref_to_offset(data)?;
        self.get_mut_slice_ref_unaligned::<T>(offset, data.len())
    }
    /// Convert an object to a mutable reference regardless of potential alignment issues.
    ///
    /// You should not do this unless you know your alignment situation. See
    /// [`Buffer::get_mut_slice_ref_unaligned`](Buffer::get_mut_slice_ref_unaligned) for an explanation as to why.
    unsafe fn force_make_mut_slice_ref<T: Castable>(&mut self, data: &[T]) -> Result<&mut [T], Error> {
        // I'm unsure why the borrow checker is annoyed at this code, attempting to go out
        // of scope of the returned error (or even explicitly dropping it) still doesn't let
        // me borrow again, so just do some pointer magic to make a new reference. if you
        // know why this is causing a borrow issue and how to fix it please file a ticket on GitHub.
        let second_ref = &mut *(self as *mut Self);
        let offset = self.slice_ref_to_offset(data)?;
        
        match self.get_mut_slice_ref::<T>(offset, data.len()) {
            Ok(ref_data) => Ok(ref_data),
            Err(err) => {
                if let Error::BadAlignment(_,_) = err { second_ref.get_mut_slice_ref_unaligned::<T>(offset, data.len()) }
                else { Err(err) }
            },
        }
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
    fn write_ref<T: Castable>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.write(offset, bytes)
    }
    /// Write a given slice object of type *T* to the given buffer at the given *offset*.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn write_slice_ref<T: Castable>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
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
    fn start_with_ref<T: Castable>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.start_with(bytes)
    }
    /// Start the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn start_with_slice_ref<T: Castable>(&mut self, data: &[T]) -> Result<(), Error> {
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
    fn end_with_ref<T: Castable>(&mut self, data: &T) -> Result<(), Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.end_with(bytes)
    }
    /// End the buffer with the given slice reference data.
    ///
    /// Returns an [`Error::OutOfBounds`](Error::OutOfBounds) error if the write runs out of boundaries.
    fn end_with_slice_ref<T: Castable>(&mut self, data: &[T]) -> Result<(), Error> {
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
    fn search_ref<'a, T: Castable>(&'a self, data: &T) -> Result<BufferSearchIter<'a>, Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        self.search(bytes)
    }
    /// Search for the following slice reference of type *T*. This converts the slice into a [`u8`](u8) [slice](slice).
    /// See [`Buffer::search`](Buffer::search).
    fn search_slice_ref<'a, T: Castable>(&'a self, data: &[T]) -> Result<BufferSearchIter<'a>, Error> {
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
    fn contains_ref<T: Castable>(&self, data: &T) -> Result<bool, Error> {
        let bytes = ref_to_bytes::<T>(data)?;
        Ok(self.contains(bytes))
    }
    /// Check if this buffer contains the following slice of type *T*.
    fn contains_slice_ref<T: Castable>(&self, data: &[T]) -> Result<bool, Error> {
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
