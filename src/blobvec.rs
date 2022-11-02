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
    pub unsafe fn add<T>(&mut self, object: T) {
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        self.data.extend_from_slice(slice);
        self.len += 1;
        std::mem::forget(object);
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
        unsafe { blob.add(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 1);
        assert_eq!(blob.data, [0, 0, 0, 0]);

        let val: u32 = 32;
        unsafe { blob.add(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);
    }

    #[test]
    fn blob_get() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.add(val) };

        let ptr = unsafe { blob.get(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;

        assert_eq!(ptr, blob.data.as_ptr());

        let val: u32 = 32;
        unsafe { blob.add(val) };

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
        unsafe { blob.add(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0]);

        let val: u32 = 32;
        unsafe { blob.add(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0, 32]);
    }
}
