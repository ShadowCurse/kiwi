use std::alloc::Layout;

#[derive(Debug)]
pub struct BlobVec {
    layout: Layout,
    len: usize,
    data: Vec<u8>,
}

impl BlobVec {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            len: 0,
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// # Safety
    /// The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn push<T>(&mut self, object: T) {
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        self.data.extend_from_slice(slice);
        self.len += 1;
        std::mem::forget(object);
    }

    /// # Safety
    /// The slice should contain data of type T that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn push_from_slice(&mut self, object_slice: &[u8]) {
        self.data.extend_from_slice(object_slice);
        self.len += 1;
    }

    /// # Safety
    /// The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn insert<T>(&mut self, index: usize, object: T) {
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        let object_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.get_mut(index) as *mut () as *mut u8,
                self.layout.size(),
            )
        };
        object_slice.copy_from_slice(slice);
        std::mem::forget(object);
    }

    /// # Safety
    /// The slice should contain data of type T that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn insert_from_slice(&mut self, index: usize, object_slice: &[u8]) {
        let object_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.get_mut(index) as *mut () as *mut u8,
                self.layout.size(),
            )
        };
        object_slice.copy_from_slice(object_slice);
    }

    /// # Safety
    /// The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get(&self, index: usize) -> &() {
        let data_index = index * self.layout.size();
        std::mem::transmute(&self.data[data_index])
    }

    /// # Safety
    /// The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_mut(&mut self, index: usize) -> &mut () {
        let data_index = index * self.layout.size();
        std::mem::transmute(&mut self.data[data_index])
    }

    /// # Safety
    /// The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_as_byte_slice(&self, index: usize) -> &[u8] {
        let data_index = index * self.layout.size();
        unsafe { std::slice::from_raw_parts(&self.data[data_index], self.layout.size()) }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// # Safety
    /// The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        assert!(
            self.layout == Layout::new::<T>(),
            "casting to type with different layout"
        );
        let ptr: *const T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts(ptr, self.len)
    }

    /// # Safety
    /// The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn as_slice_mut<T>(&mut self) -> &mut [T] {
        assert!(
            self.layout == Layout::new::<T>(),
            "casting to type with different layout"
        );
        let ptr: *mut T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts_mut(ptr, self.len)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blob_new() {
        let layout = Layout::new::<u32>();
        let blob = BlobVec::new(layout);
        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 0);
        assert_eq!(blob.data, []);
    }

    #[test]
    fn blob_add() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.push(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 1);
        assert_eq!(blob.data, [0, 0, 0, 0]);

        let val: u32 = 32;
        unsafe { blob.push(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);

        let val: [u8; 4] = [69, 0, 0, 0];
        unsafe { blob.push_from_slice(&val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0, 69, 0, 0, 0]);
    }

    #[test]
    fn blob_insert() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.push(val) };
        let val: u32 = 32;
        unsafe { blob.push(val) };

        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);

        let val = 69;
        unsafe { blob.insert(1, val) };

        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 69, 0, 0, 0]);

        let val: [u8; 4] = [11, 0, 0, 0];
        unsafe { blob.insert_from_slice(0, &val) };

        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [11, 0, 0, 0, 69, 0, 0, 0]);
    }

    #[test]
    fn blob_get_and_add_from_slice() {
        #[derive(PartialEq, Eq)]
        struct Foo {
            a: u32,
            b: bool,
            c: (u8, u8),
        }
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout);

        let val = Foo {
            a: 1,
            b: true,
            c: (6, 9),
        };
        unsafe { blob.push(val) };
        assert_eq!(blob.data, [1, 0, 0, 0, 1, 6, 9, 0]);

        let val_as_slice = unsafe { blob.get_as_byte_slice(0) };
        assert_eq!(val_as_slice, [1, 0, 0, 0, 1, 6, 9, 0]);

        let mut blob = BlobVec::new(layout);
        unsafe { blob.push_from_slice(val_as_slice) };
        assert_eq!(blob.data, [1, 0, 0, 0, 1, 6, 9, 0]);
    }

    #[test]
    fn blob_get() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;

        assert_eq!(ptr, blob.data.as_ptr());

        let val: u32 = 32;
        unsafe { blob.push(val) };

        let ptr = unsafe { blob.get(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, blob.data.as_ptr());
        let ptr = unsafe { blob.get(1) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, &blob.data[4] as *const u8);
    }

    #[test]
    fn blob_as_slice() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

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
