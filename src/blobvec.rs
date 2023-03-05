use std::alloc::Layout;

#[derive(Debug)]
pub struct BlobVec {
    layout: Layout,
    data: Vec<u8>,
    free_slot: Box<[u8]>,
    drop: Option<fn(*mut ())>,
}

impl BlobVec {
    pub fn new(layout: Layout, drop: Option<fn(*mut ())>) -> Self {
        Self {
            layout,
            data: Vec::new(),
            free_slot: vec![0; layout.size()].into(),
            drop,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() / self.layout.size()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Pushes `object` to the end of the [`BlobVec`]
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn push<T>(&mut self, object: T) {
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        self.data.extend_from_slice(slice);
        std::mem::forget(object);
    }

    /// Pushes `object` respresented as slice of bytes to the end of the [`BlobVec`]
    ///
    /// # Safety
    /// - The slice should contain data of type T that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn push_from_slice(&mut self, object: &[u8]) {
        self.data.extend_from_slice(object);
    }

    /// The slice should contain data of type T that is stored inside the [`BlobVec`]
    #[inline]
    pub fn push_empty(&mut self) {
        self.data.extend_from_slice(&self.free_slot);
    }

    /// Overwrites the object at `index` with new object
    /// Does not drop the overwritten object
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    /// - Index should be in range `0..BlobVec::len`
    #[inline]
    pub unsafe fn overwrite<T>(&mut self, index: usize, object: T) {
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        let object_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.get_erased_ref_mut(index) as *mut () as *mut u8,
                self.layout.size(),
            )
        };
        object_slice.copy_from_slice(slice);
        std::mem::forget(object);
    }

    /// Overwrites the object at `index` with new object
    /// represented as slice of bytes
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    /// - Index should be in range `0..BlobVec::len`
    #[inline]
    pub unsafe fn overwrite_from_slice(&mut self, index: usize, object: &[u8]) {
        let object_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.get_erased_ref_mut(index) as *mut () as *mut u8,
                self.layout.size(),
            )
        };
        object_slice.copy_from_slice(object);
    }

    /// Get a reference to the object at `index`
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get<T>(&self, index: usize) -> &T {
        let data_index = index * self.layout.size();
        std::mem::transmute(&self.data[data_index])
    }

    /// Get a mutable reference to the object at `index`
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_mut<T>(&mut self, index: usize) -> &mut T {
        let data_index = index * self.layout.size();
        std::mem::transmute(&mut self.data[data_index])
    }

    /// Get a reference to the object at `index`
    /// as a reference to void
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_erased_ref(&self, index: usize) -> &() {
        let data_index = index * self.layout.size();
        std::mem::transmute(&self.data[data_index])
    }

    /// Get a mutable reference to the object at `index`
    /// as a mutable reference to void
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_erased_ref_mut(&mut self, index: usize) -> &mut () {
        let data_index = index * self.layout.size();
        std::mem::transmute(&mut self.data[data_index])
    }

    /// Get a reference to the object at `index`
    /// as a const pointer to void
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_erased_ptr(&self, index: usize) -> *const () {
        let data_index = index * self.layout.size();
        &self.data[data_index] as *const u8 as *const ()
    }

    /// Get a mutable reference to the object at `index`
    /// as a mutable pointer to void
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_erased_ptr_mut(&self, index: usize) -> *mut () {
        let data_index = index * self.layout.size();
        &self.data[data_index] as *const u8 as *mut ()
    }

    /// Get an object at `index` as a refernce to
    /// the slice of byte
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_as_byte_slice(&self, index: usize) -> &[u8] {
        let data_index = index * self.layout.size();
        unsafe { std::slice::from_raw_parts(&self.data[data_index], self.layout.size()) }
    }

    /// Drops the object at `index`
    ///
    /// # Safety
    /// - If drop is not `None`, the object should not be already dropped
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn drop_at(&mut self, index: usize) {
        if let Some(drop) = self.drop {
            drop(self.get_erased_ptr_mut(index));
        }
    }

    /// Swaps the object at `index` with new object
    /// Returns the swapped object
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn swap<T>(&mut self, index: usize, mut object: T) -> T {
        std::mem::swap(&mut object, self.get_mut(index));
        object
    }

    /// Returns a mutable pointer to the internal data
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Reinterprets internal data storage as a refernce to the slice of type `T`
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        assert!(
            self.layout == Layout::new::<T>(),
            "casting to type with different layout"
        );
        let ptr: *const T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Reinterprets internal data storage as a mutable refernce to the slice of type `T`
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn as_slice_mut<T>(&mut self) -> &mut [T] {
        assert!(
            self.layout == Layout::new::<T>(),
            "casting to type with different layout"
        );
        let ptr: *mut T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts_mut(ptr, self.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blob_new() {
        let layout = Layout::new::<u32>();
        let blob = BlobVec::new(layout, None);
        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.data, []);
    }

    #[test]
    fn blob_push() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 0;
        unsafe { blob.push(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.data, [0, 0, 0, 0]);

        let val: u32 = 32;
        unsafe { blob.push(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len(), 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);

        let val: [u8; 4] = [69, 0, 0, 0];
        unsafe { blob.push_from_slice(&val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0, 69, 0, 0, 0]);
    }

    #[test]
    fn blob_insert() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 0;
        unsafe { blob.push(val) };
        let val: u32 = 32;
        unsafe { blob.push(val) };

        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);

        let val = 69;
        unsafe { blob.overwrite(1, val) };

        assert_eq!(blob.len(), 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 69, 0, 0, 0]);

        let val: [u8; 4] = [11, 0, 0, 0];
        unsafe { blob.overwrite_from_slice(0, &val) };

        assert_eq!(blob.len(), 2);
        assert_eq!(blob.data, [11, 0, 0, 0, 69, 0, 0, 0]);
    }

    #[test]
    fn blob_get_and_push_from_slice() {
        #[derive(PartialEq, Eq)]
        struct Foo {
            a: u32,
            b: bool,
            c: (u8, u8),
        }
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        let val = Foo {
            a: 1,
            b: true,
            c: (6, 9),
        };
        unsafe { blob.push(val) };

        let val_as_slice = unsafe { blob.get_as_byte_slice(0) };
        assert_eq!(val_as_slice, blob.data);

        let mut new_blob = BlobVec::new(layout, None);
        unsafe { new_blob.push_from_slice(val_as_slice) };
        assert_eq!(new_blob.data, blob.data);
    }

    #[test]
    fn blob_get_erased_ref() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 0;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get_erased_ref(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;

        assert_eq!(ptr, blob.data.as_ptr());

        let val: u32 = 32;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get_erased_ref(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, blob.data.as_ptr());
        let ptr = unsafe { blob.get_erased_ref(1) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, &blob.data[4] as *const u8);
    }

    #[test]
    fn blob_get_erased_ptr() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 0;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get_erased_ptr(0) };
        let ptr: *const u8 = ptr as *const u8;

        assert_eq!(ptr, blob.data.as_ptr());

        let val: u32 = 32;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get_erased_ptr(0) };
        let ptr: *const u8 = ptr as *const u8;
        assert_eq!(ptr, blob.data.as_ptr());
        let ptr = unsafe { blob.get_erased_ptr(1) };
        let ptr: *const u8 = ptr as *const u8;
        assert_eq!(ptr, &blob.data[4] as *const u8);
    }

    #[test]
    fn blob_as_slice() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 0;
        unsafe { blob.push(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0]);

        let val: u32 = 32;
        unsafe { blob.push(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0, 32]);
    }
}
