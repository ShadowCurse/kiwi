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
        let old_object_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.get_erased_ptr_mut(index).cast::<u8>(),
                self.layout.size(),
            )
        };
        old_object_slice.copy_from_slice(slice);
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

    /// Get a mutable reference to the object at `index`
    /// from shared reference
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    /// - Same index should not be borrowed more then once
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_unchecked<T>(&self, index: usize) -> &mut T {
        &mut *(self.get_erased_ptr_mut(index) as *mut T)
    }

    /// Get a pointer to the object at `index`
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_ptr<T>(&self, index: usize) -> *const T {
        let data_index = index * self.layout.size();
        &self.data[data_index] as *const u8 as *const T
    }

    /// Get a mutable reference to the object at `index`
    ///
    /// # Safety
    /// - The index should be in range 0 to blobvec.len()
    #[inline]
    pub unsafe fn get_ptr_mut<T>(&self, index: usize) -> *mut T {
        let data_index = index * self.layout.size();
        &self.data[data_index] as *const u8 as *mut T
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
        let ptr: *const T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Reinterprets internal data storage as a mutable refernce to the slice of type `T`
    ///
    /// # Safety
    /// - The type T should be the type that is stored inside the [`BlobVec`]
    #[inline]
    pub unsafe fn as_slice_mut<T>(&mut self) -> &mut [T] {
        let ptr: *mut T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts_mut(ptr, self.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Foo {
        a: u32,
        b: bool,
        c: (u8, u8),
    }

    #[test]
    fn blob_new_u32() {
        let layout = Layout::new::<u32>();
        let blob = BlobVec::new(layout, None);
        assert_eq!(blob.layout, layout);
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.data, []);
    }

    #[test]
    fn blob_new_foo() {
        let layout = Layout::new::<Foo>();
        let blob = BlobVec::new(layout, None);
        assert_eq!(blob.layout, layout);
        assert_eq!(blob.len(), 0);
        assert_eq!(blob.data, []);
    }

    #[test]
    fn blob_push_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        let val: u32 = 32;
        unsafe { blob.push(val) };
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.data, [32, 0, 0, 0]);
        let val_ref: &u32 = unsafe { blob.get(0) };
        assert_eq!(val_ref, &val);

        let val: [u8; 4] = [69, 0, 0, 0];
        unsafe { blob.push_from_slice(&val) };
        assert_eq!(blob.len(), 2);
        assert_eq!(blob.data, [32, 0, 0, 0, 69, 0, 0, 0]);
        let val_ref: &u32 = unsafe { blob.get(1) };
        assert_eq!(val_ref, &69);

        blob.push_empty();
        assert_eq!(blob.len(), 3);
        assert_eq!(blob.data, [32, 0, 0, 0, 69, 0, 0, 0, 0, 0, 0, 0]);
        let val_ref: &u8 = unsafe { blob.get(2) };
        assert_eq!(val_ref, &0);
    }

    #[test]
    fn blob_push_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        let val = Foo {
            a: 69,
            b: true,
            c: (6, 9),
        };
        unsafe { blob.push(val) };
        assert_eq!(blob.len(), 1);
        assert_eq!(blob.data, [69, 0, 0, 0, 6, 9, 1, 0]);
        assert_eq!(blob.data.len(), std::mem::size_of::<Foo>());
        let val_ref: &Foo = unsafe { blob.get(0) };
        assert_eq!(val_ref, &val);

        let val: [u8; 8] = [11, 0, 0, 0, 1, 1, 0, 0];
        unsafe { blob.push_from_slice(&val) };
        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.data,
            [69, 0, 0, 0, 6, 9, 1, 0, 11, 0, 0, 0, 1, 1, 0, 0]
        );
        let val_ref: &Foo = unsafe { blob.get(1) };
        assert_eq!(
            val_ref,
            &Foo {
                a: 11,
                b: false,
                c: (1, 1)
            }
        );

        blob.push_empty();
        assert_eq!(blob.len(), 3);
        assert_eq!(
            blob.data,
            [69, 0, 0, 0, 6, 9, 1, 0, 11, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        let val_ref: &Foo = unsafe { blob.get(2) };
        assert_eq!(
            val_ref,
            &Foo {
                a: 0,
                b: false,
                c: (0, 0)
            }
        );
    }

    #[test]
    fn blob_overwrite_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(32) };
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
    #[rustfmt::skip]
    fn blob_overwrite_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 69, b: true,  c: (6, 9) }) };
        unsafe { blob.push(Foo { a: 32, b: false, c: (3, 2) }) };
        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.data,
            [69, 0, 0, 0, 6, 9, 1, 0, 32, 0, 0, 0, 3, 2, 0, 0]
        );

        let val = Foo {
            a: 11,
            b: true,
            c: (1, 1),
        };
        unsafe { blob.overwrite(1, val) };
        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.data,
            [69, 0, 0, 0, 6, 9, 1, 0, 11, 0, 0, 0, 1, 1, 1, 0]
        );

        let val: [u8; 8] = [22, 0, 0, 0, 2, 2, 0, 0];
        unsafe { blob.overwrite_from_slice(0, &val) };
        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.data,
            [22, 0, 0, 0, 2, 2, 0, 0, 11, 0, 0, 0, 1, 1, 1, 0]
        );
    }

    #[test]
    fn blob_get_and_push_from_slice() {
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
        assert_eq!(val_as_slice.len(), std::mem::size_of::<Foo>());

        let mut new_blob = BlobVec::new(layout, None);
        unsafe { new_blob.push_from_slice(val_as_slice) };
        assert_eq!(new_blob.data, blob.data);
    }

    #[test]
    fn blob_get_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(1) };
        unsafe { blob.push(2) };
        unsafe { blob.push(3) };

        assert_eq!(unsafe { blob.get::<u32>(0) }, &0);
        assert_eq!(unsafe { blob.get::<u32>(1) }, &1);
        assert_eq!(unsafe { blob.get::<u32>(2) }, &2);
        assert_eq!(unsafe { blob.get::<u32>(3) }, &3);

        assert_eq!(unsafe { blob.get_mut::<u32>(0) }, &mut 0);
        assert_eq!(unsafe { blob.get_mut::<u32>(1) }, &mut 1);
        assert_eq!(unsafe { blob.get_mut::<u32>(2) }, &mut 2);
        assert_eq!(unsafe { blob.get_mut::<u32>(3) }, &mut 3);

        assert_eq!(unsafe { blob.get_mut_unchecked::<u32>(0) }, &mut 0);
        assert_eq!(unsafe { blob.get_mut_unchecked::<u32>(1) }, &mut 1);
        assert_eq!(unsafe { blob.get_mut_unchecked::<u32>(2) }, &mut 2);
        assert_eq!(unsafe { blob.get_mut_unchecked::<u32>(3) }, &mut 3);
    }

    #[test]
    #[rustfmt::skip]
    fn blob_get_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
        unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
        unsafe { blob.push(Foo { a: 2, b: false, c: (2, 2) }) };
        unsafe { blob.push(Foo { a: 3, b: false, c: (3, 3) }) };

        assert_eq!(unsafe { blob.get::<Foo>(0) }, &Foo { a: 0, b: false, c: (0, 0) });
        assert_eq!(unsafe { blob.get::<Foo>(1) }, &Foo { a: 1, b: false, c: (1, 1) });
        assert_eq!(unsafe { blob.get::<Foo>(2) }, &Foo { a: 2, b: false, c: (2, 2) });
        assert_eq!(unsafe { blob.get::<Foo>(3) }, &Foo { a: 3, b: false, c: (3, 3) });

        assert_eq!(unsafe { blob.get_mut::<Foo>(0) }, &Foo { a: 0, b: false, c: (0, 0) });
        assert_eq!(unsafe { blob.get_mut::<Foo>(1) }, &Foo { a: 1, b: false, c: (1, 1) });
        assert_eq!(unsafe { blob.get_mut::<Foo>(2) }, &Foo { a: 2, b: false, c: (2, 2) });
        assert_eq!(unsafe { blob.get_mut::<Foo>(3) }, &Foo { a: 3, b: false, c: (3, 3) });

        assert_eq!(unsafe { blob.get_mut_unchecked::<Foo>(0) }, &Foo { a: 0, b: false, c: (0, 0) });
        assert_eq!(unsafe { blob.get_mut_unchecked::<Foo>(1) }, &Foo { a: 1, b: false, c: (1, 1) });
        assert_eq!(unsafe { blob.get_mut_unchecked::<Foo>(2) }, &Foo { a: 2, b: false, c: (2, 2) });
        assert_eq!(unsafe { blob.get_mut_unchecked::<Foo>(3) }, &Foo { a: 3, b: false, c: (3, 3) });
    }

    #[test]
    fn blob_get_ptr_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(1) };
        unsafe { blob.push(2) };
        unsafe { blob.push(3) };

        assert_eq!(unsafe { *blob.get_ptr::<u32>(0) }, 0);
        assert_eq!(unsafe { *blob.get_ptr::<u32>(1) }, 1);
        assert_eq!(unsafe { *blob.get_ptr::<u32>(2) }, 2);
        assert_eq!(unsafe { *blob.get_ptr::<u32>(3) }, 3);

        assert_eq!(unsafe { *blob.get_ptr_mut::<u32>(0) }, 0);
        assert_eq!(unsafe { *blob.get_ptr_mut::<u32>(1) }, 1);
        assert_eq!(unsafe { *blob.get_ptr_mut::<u32>(2) }, 2);
        assert_eq!(unsafe { *blob.get_ptr_mut::<u32>(3) }, 3);
    }

    #[test]
    #[rustfmt::skip]
    fn blob_get_ptr_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
        unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
        unsafe { blob.push(Foo { a: 2, b: false, c: (2, 2) }) };
        unsafe { blob.push(Foo { a: 3, b: false, c: (3, 3) }) };

        assert_eq!(unsafe { *blob.get_ptr::<Foo>(0) }, Foo { a: 0, b: false, c: (0, 0) });
        assert_eq!(unsafe { *blob.get_ptr::<Foo>(1) }, Foo { a: 1, b: false, c: (1, 1) });
        assert_eq!(unsafe { *blob.get_ptr::<Foo>(2) }, Foo { a: 2, b: false, c: (2, 2) });
        assert_eq!(unsafe { *blob.get_ptr::<Foo>(3) }, Foo { a: 3, b: false, c: (3, 3) });

        assert_eq!(unsafe { *blob.get_mut::<Foo>(0) }, Foo { a: 0, b: false, c: (0, 0) });
        assert_eq!(unsafe { *blob.get_mut::<Foo>(1) }, Foo { a: 1, b: false, c: (1, 1) });
        assert_eq!(unsafe { *blob.get_mut::<Foo>(2) }, Foo { a: 2, b: false, c: (2, 2) });
        assert_eq!(unsafe { *blob.get_mut::<Foo>(3) }, Foo { a: 3, b: false, c: (3, 3) });
    }

    #[test]
    fn blob_get_erased_ref_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(1) };
        unsafe { blob.push(2) };
        unsafe { blob.push(3) };

        let check_ref = |index: u32| {
            let reference = unsafe { blob.get_erased_ref(index as usize) };
            let reference = unsafe { &*(reference as *const () as *const u32) };
            assert_eq!(reference, &index);
        };

        check_ref(0);
        check_ref(1);
        check_ref(2);
        check_ref(3);

        let mut check_ref_mut = |mut index: u32| {
            let reference = unsafe { blob.get_erased_ref_mut(index as usize) };
            let reference = unsafe { &*(reference as *mut () as *mut u32) };
            assert_eq!(reference, &mut index);
        };

        check_ref_mut(0);
        check_ref_mut(1);
        check_ref_mut(2);
        check_ref_mut(3);
    }

    #[test]
    #[rustfmt::skip]
    fn blob_get_erased_ref_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
        unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
        unsafe { blob.push(Foo { a: 2, b: false, c: (2, 2) }) };
        unsafe { blob.push(Foo { a: 3, b: false, c: (3, 3) }) };

        let check_ref = |index: u32| {
            let reference = unsafe { blob.get_erased_ref(index as usize) };
            let reference = unsafe { &*(reference as *const () as *const Foo) };
            assert_eq!(reference, &Foo { a: index, b: false, c: (index as u8, index as u8) });
        };

        check_ref(0);
        check_ref(1);
        check_ref(2);
        check_ref(3);

        let mut check_ref_mut = |index: u32| {
            let reference = unsafe { blob.get_erased_ref_mut(index as usize) };
            let reference = unsafe { &*(reference as *mut () as *mut Foo) };
            assert_eq!(reference, &mut Foo { a: index, b: false, c: (index as u8, index as u8) });
        };

        check_ref_mut(0);
        check_ref_mut(1);
        check_ref_mut(2);
        check_ref_mut(3);
    }

    #[test]
    fn blob_get_erased_ptr_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(1) };
        unsafe { blob.push(2) };
        unsafe { blob.push(3) };

        let check_ref = |index: u32| {
            let reference = unsafe { blob.get_erased_ptr(index as usize) };
            let reference = unsafe { &*(reference as *const u32) };
            assert_eq!(reference, &index);
        };

        check_ref(0);
        check_ref(1);
        check_ref(2);
        check_ref(3);

        let check_ref_mut = |mut index: u32| {
            let reference = unsafe { blob.get_erased_ptr_mut(index as usize) };
            let reference = unsafe { &*(reference as *mut u32) };
            assert_eq!(reference, &mut index);
        };

        check_ref_mut(0);
        check_ref_mut(1);
        check_ref_mut(2);
        check_ref_mut(3);
    }

    #[test]
    #[rustfmt::skip]
    fn blob_get_erased_ptr_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
        unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
        unsafe { blob.push(Foo { a: 2, b: false, c: (2, 2) }) };
        unsafe { blob.push(Foo { a: 3, b: false, c: (3, 3) }) };

        let check_ref = |index: u32| {
            let reference = unsafe { blob.get_erased_ptr(index as usize) };
            let reference = unsafe { &*(reference as *const Foo) };
            assert_eq!(reference, &Foo { a: index, b: false, c: (index as u8, index as u8) });
        };

        check_ref(0);
        check_ref(1);
        check_ref(2);
        check_ref(3);

        let check_ref_mut = |index: u32| {
            let reference = unsafe { blob.get_erased_ptr_mut(index as usize) };
            let reference = unsafe { &*(reference as *mut Foo) };
            assert_eq!(reference, &mut Foo { a: index, b: false, c: (index as u8, index as u8) });
        };

        check_ref_mut(0);
        check_ref_mut(1);
        check_ref_mut(2);
        check_ref_mut(3);
    }

    #[test]
    fn blob_get_as_byte_slice_u32() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(0) };
        unsafe { blob.push(1) };
        unsafe { blob.push(2) };
        unsafe { blob.push(3) };

        let check_byte_slice = |index: u8| {
            let reference = unsafe { blob.get_as_byte_slice(index as usize) };
            assert_eq!(reference, &[index, 0, 0, 0]);
        };

        check_byte_slice(0);
        check_byte_slice(1);
        check_byte_slice(2);
        check_byte_slice(3);
    }

    #[test]
    #[rustfmt::skip]
    fn blob_get_as_byte_slice_foo() {
        let layout = Layout::new::<Foo>();
        let mut blob = BlobVec::new(layout, None);

        unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
        unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
        unsafe { blob.push(Foo { a: 2, b: false, c: (2, 2) }) };
        unsafe { blob.push(Foo { a: 3, b: false, c: (3, 3) }) };

        let check_byte_slice = |index: u8| {
            let reference = unsafe { blob.get_as_byte_slice(index as usize) };
            assert_eq!(reference, &[index, 0, 0, 0, index, index, 0, 0]);
        };

        check_byte_slice(0);
        check_byte_slice(1);
        check_byte_slice(2);
        check_byte_slice(3);
    }

    #[test]
    fn blob_drop_at() {
        use std::rc::Rc;

        struct A {
            _a: u32,
        }
        type T = Rc<A>;

        fn type_drop(component: *mut ()) {
            unsafe { component.cast::<T>().drop_in_place() };
        }

        let layout = Layout::new::<T>();
        let mut blob = BlobVec::new(layout, Some(type_drop));

        let val = Rc::new(A { _a: 1 });
        let val_copy = val.clone();
        let val_copy_2 = val.clone();
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);

        unsafe { blob.push(val) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);
        unsafe { blob.push(val_copy) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);

        unsafe { blob.drop_at(0) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 2);
        unsafe { blob.drop_at(1) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 1);
    }

    #[test]
    fn blob_swap() {
        use std::rc::Rc;

        struct A {
            _a: u32,
        }
        type T = Rc<A>;

        let layout = Layout::new::<T>();
        let mut blob = BlobVec::new(layout, None);

        let val = Rc::new(A { _a: 1 });
        let val_copy = val.clone();
        let val_copy_2 = val.clone();
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);

        unsafe { blob.push(val) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);
        unsafe { blob.push(val_copy) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_2), 3);

        let val_swapped = unsafe { blob.swap(0, val_copy_2) };
        assert_eq!(Rc::<A>::strong_count(&val_swapped), 3);
        let val_copy_swapped = unsafe { blob.swap(1, val_swapped) };
        assert_eq!(Rc::<A>::strong_count(&val_copy_swapped), 3);
    }

    #[test]
    fn blob_as_mut_ptr() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout, None);
        unsafe { blob.push(69) };
        assert_eq!(blob.as_mut_ptr(), blob.data.as_mut_ptr());
    }

    #[test]
    fn blob_as_slice_u32() {
        {
            let layout = Layout::new::<u32>();
            let mut blob = BlobVec::new(layout, None);

            unsafe { blob.push(0) };
            let slice = unsafe { blob.as_slice::<u32>() };
            assert_eq!(slice, &[0]);

            unsafe { blob.push(32) };
            let slice = unsafe { blob.as_slice::<u32>() };
            assert_eq!(slice, &[0, 32]);
        }
        {
            let layout = Layout::new::<u32>();
            let mut blob = BlobVec::new(layout, None);

            unsafe { blob.push(0) };
            let slice = unsafe { blob.as_slice_mut::<u32>() };
            assert_eq!(slice, &mut [0]);

            unsafe { blob.push(32) };
            let slice = unsafe { blob.as_slice_mut::<u32>() };
            assert_eq!(slice, &mut [0, 32]);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn blob_as_slice_foo() {
        {
            let layout = Layout::new::<Foo>();
            let mut blob = BlobVec::new(layout, None);

            unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
            let slice = unsafe { blob.as_slice::<Foo>() };
            assert_eq!(slice, &[Foo { a: 0, b: false, c: (0, 0) }]);

            unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
            let slice = unsafe { blob.as_slice::<Foo>() };
            assert_eq!(slice, &[Foo { a: 0, b: false, c: (0, 0) }, Foo { a: 1, b: false, c: (1, 1) }]);
        }
        {
            let layout = Layout::new::<Foo>();
            let mut blob = BlobVec::new(layout, None);

            unsafe { blob.push(Foo { a: 0, b: false, c: (0, 0) }) };
            let slice = unsafe { blob.as_slice_mut::<Foo>() };
            assert_eq!(slice, &mut [Foo { a: 0, b: false, c: (0, 0) }]);

            unsafe { blob.push(Foo { a: 1, b: false, c: (1, 1) }) };
            let slice = unsafe { blob.as_slice_mut::<Foo>() };
            assert_eq!(slice, &mut [Foo { a: 0, b: false, c: (0, 0) }, Foo { a: 1, b: false, c: (1, 1) }]);
        }
    }
}
