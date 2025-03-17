use std::mem::MaybeUninit;
use std::ptr;

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

/* impl<T, const CAP: usize> Index<usize> for SHSTBuffer<T, CAP> {
    type Output = MaybeUninit<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, const CAP: usize> IndexMut<usize> for SHSTBuffer<T, CAP> {
    type Output = MaybeUninit<T>;

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
} */

/* pub fn try_push_str<'a>(&mut self, s: &'a str) -> Result<(), CapacityError<&'a str>> {
    if s.len() > self.capacity() - self.len() {
        return Err(CapacityError::new(s));
    }
    unsafe {
        let dst = self.as_mut_ptr().add(self.len());
        let src = s.as_ptr();
        ptr::copy_nonoverlapping(src, dst, s.len());
        let newl = self.len() + s.len();
        self.set_len(newl);
    }
    Ok(())
} */

pub fn stack_to_heap_pumping_1() {
    let chunk: &[u8] = "chunkongoongoongoongoongoongong".as_bytes();
    let mut stack_buffer: SHSTBuffer<u8, { (1024 * 32) + 1 }> = SHSTBuffer::new();

    let src = chunk.as_ptr();
    let mut dst = stack_buffer.data.as_mut_ptr() as *mut u8;

    for i in 0..1024 {
        unsafe {
            ptr::copy_nonoverlapping(src, dst, chunk.len());
            dst = dst.add(chunk.len());
            *dst = (i % 255) as u8;
        }
    }
}
