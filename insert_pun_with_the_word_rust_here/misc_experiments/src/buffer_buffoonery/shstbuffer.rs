use std::mem::MaybeUninit;
use std::ops::Index;

/**
 * a SHitty STack Buffer. obviously there's already a library for this, but
 * a simple implementation allows me to get a better look at what happens
 */
pub struct SHSTBuffer<T, const CAP: usize> {
    pub data: [MaybeUninit<T>; CAP],
}

impl<T, const CAP: usize> SHSTBuffer<T, CAP> {
    pub fn new() -> Self {
        unsafe {
            Self {
                data: MaybeUninit::uninit().assume_init(),
            }
        }
    }
}

pub fn stack_to_heap_pumping_1() {
    let buffer: SHSTBuffer<i32, 1024> = SHSTBuffer::new();
}
