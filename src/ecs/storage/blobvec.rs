use std::{ptr::NonNull, alloc::Layout};



pub struct BlobVec {
    item_layout: Layout, // layout of the item in the vec
    data: NonNull<u8>, // 1 byte blocks, layout is handled manually
    capacity: usize, // capacity of the vec in bytes (index capacity)
    len: usize, // number of used bytes in the vec (used indices) (an item has layout.size amount of bytes)
    swap_space: NonNull<u8>, // a single item space for swap operation
    drop: unsafe fn(*mut u8), // drop function to use when removing items, not the blobvec itself
}

impl BlobVec {
    pub fn new(item_layout: Layout, capacity: usize, drop: unsafe fn(*mut u8)) -> BlobVec {
        if item_layout.size() == 0 { // for 0 size marker structs (ZST: Zero Sized Type)
            BlobVec {
                item_layout,
                data: NonNull::dangling(),
                capacity: usize::MAX, // Max capacity from the let go for ZSTs
                len: 0,
                swap_space: NonNull::dangling(), // No need for a swap space for ZSTs
                drop,
            }
        }
        else {
            let swap = NonNull::new(unsafe { std::alloc::alloc(item_layout) })
                .unwrap_or_else(|| std::alloc::handle_alloc_error(item_layout));
            let mut blobvec = BlobVec {
                item_layout,
                data: NonNull::dangling(),
                capacity: 0,
                len: 0,
                swap_space: swap,
                drop,
            };
            blobvec.reserve_exact(capacity);
            blobvec
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn item_capacity(&self) -> usize {
        self.capacity / self.item_layout.size()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn items_len(&self) -> usize {
        self.len / self.item_layout.size()
    }

    pub fn get_ptr(&self) -> NonNull<u8> {
        self.data
    }
    
    // Reserves (amount indices) space for future usage
    // Grow in size (alloc new space) if capacity is not enough
    pub fn reserve_exact(&mut self, additional: usize) {
        let available = self.capacity - self.len;
        if available < additional {
            self.grow_exact(additional - available);
        }
    }

    // Grow (amount indices) in space
    // new_capacity = capacity + amount
    // new_size_in_bytes = size_in_bytes + (amount * item_layout.size())
    fn grow_exact(&mut self, additional: usize) {
        debug_assert!(self.item_layout.size() != 0, "No need to and cannot grow a ZST vec");
        
        let new_capacity = self.capacity + additional;
        let new_layout = array_layout(self.item_layout, new_capacity)
                                    .expect("Invalid array layout");
        unsafe {
            let grow_data = if self.capacity == 0 {
                std::alloc::alloc(new_layout)
            }
            else {
                std::alloc::realloc(self.get_ptr().as_ptr(),
                    array_layout(self.item_layout, self.capacity)
                            .expect("Invalid array layout"),
                    new_layout.size())
            };

            self.data = NonNull::new(grow_data).unwrap_or_else(|| std::alloc::handle_alloc_error(new_layout));
        }

        self.capacity = new_capacity;
    }

    // push a 
    pub fn push_uninit(&mut self) -> usize {
        self.reserve_exact(1);
        self.len += 1;
        self.len()-1
    }

    pub unsafe fn init_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len);
        let data_i = self.get_unchecked(index);
        std::ptr::copy_nonoverlapping(value, data_i, self.item_layout.size());
    }

    pub unsafe fn replace_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len);
        let data_i = self.get_unchecked(index);
        
        let len_save = std::mem::replace(&mut self.len, 0);
        (self.drop)(data_i);
        std::ptr::copy_nonoverlapping(value, data_i, self.item_layout.size());
        self.len = len_save;
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        self.get_ptr().as_ptr().add(index * self.item_layout.size())
    }

    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        let last = self.len - 1;
        let swap_space = self.swap_space.as_ptr();
        std::ptr::copy_nonoverlapping(
            self.get_unchecked(index), 
            swap_space,
            self.item_layout.size()
        );
        std::ptr::copy(
            self.get_unchecked(last),
            self.get_unchecked(index),
            self.item_layout.size()
        );
        self.len -= 1;
        swap_space
    }

    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let swap = self.swap_remove_and_forget_unchecked(index);
        (self.drop)(swap);
    }

    pub fn clear(&mut self) {
        let len_save = self.len;
        self.len = 0;
        for i in 0..len_save {
            unsafe {
                let data_i = self.data.as_ptr().add(i * self.item_layout.size());
                (self.drop)(data_i);
            }
        }
    }

}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        let arr_layout = array_layout(self.item_layout, self.capacity)
                                    .expect("Invalid array layout");
        if arr_layout.size() > 0 {
            unsafe {
                std::alloc::dealloc(self.data.as_ptr(), arr_layout);
                std::alloc::dealloc(self.swap_space.as_ptr(), self.item_layout);
            }
        }
    }
}

fn array_layout(layout: Layout, n: usize) -> Option<Layout> {
    let (arr_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(offset, layout.size());
    Some(arr_layout)
}

// Unstable feature in Layout
// Layout::repeat(&self: Layout, n: usize) -> Result<Layout, LayoutError>
fn repeat_layout(layout: Layout, n: usize) -> Option<(Layout, usize)> {    
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;
    
    // SAFETY: self.align is already known to be valid and alloc_size has been
    // padded already.
    unsafe { Some((Layout::from_size_align_unchecked(alloc_size, layout.align()), padded_size)) }
}

// Unstable feature in Layout
// Layout::padding_needed_for(&self: Layout, align: usize) -> usize
fn padding_needed_for(layout: Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}