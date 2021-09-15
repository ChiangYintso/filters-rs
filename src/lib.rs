mod bindings;

use bindings::*;
use std::ptr::NonNull;
use std::alloc::Layout;
use std::cmp::max;

const BITS_PER_KEY: usize = 10;

/// Compute filter filter size (in both bits and bytes)
/// For small n, we can see a very high false positive rate.  Fix it
/// by enforcing a minimum filter filter length.
#[inline]
fn calc_bytes(num_keys: usize) -> usize {
    let bits = num_keys * BITS_PER_KEY;
    max((bits / 8 + 1 + 64) & (usize::MAX - 63), 64)
}

pub struct BlockedBloomFilter {
    data: NonNull<block_t>,
    layout: Layout,
    mask: usize,
}

impl BlockedBloomFilter {
    pub fn create_filter(num_keys: usize) -> BlockedBloomFilter {
        let layout = Layout::from_size_align(calc_bytes(num_keys), 64).unwrap();
        let data = NonNull::<block_t>::new(unsafe {
            std::alloc::alloc_zeroed(layout) as _
        }).unwrap();

        let mask = unsafe {
            bf_calc_num_blocks(layout.size() as _)
        };
        BlockedBloomFilter {
            data,
            layout,
            mask: mask as usize,
        }
    }

    pub fn add(&mut self, h: u32) {
        unsafe {
            bbf_add_key(h, self.data.as_mut(), self.mask as _);
        }
    }

    pub fn may_contain(&self, h: u32) -> bool {
        unsafe {
            bbf_find(h, self.data.as_ptr(), self.mask as _)
        }
    }
}

impl Drop for BlockedBloomFilter {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.data.as_mut() as *mut u64 as *mut u8, self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BlockedBloomFilter;
    use rand::Rng;

    fn rand_hashes(size: usize) -> Vec<u32> {
        let mut hashes: Vec<u32> = Vec::with_capacity(size);
        let mut rng = rand::thread_rng();
        rng.fill(hashes.as_mut_slice());
        hashes
    }

    #[test]
    fn test_filter() {
        let mut bbf = BlockedBloomFilter::create_filter(123);
        assert!(!bbf.may_contain(12345));
        bbf.add(12345);
        assert!(bbf.may_contain(12345));
    }

    #[test]
    fn test_contain_key() {
        let mut filter = BlockedBloomFilter::create_filter(10);
        let hashs = rand_hashes(10);
        for &h in &hashs {
            filter.add(h);
        }
        for h in hashs {
            assert!(filter.may_contain(h));
        }
    }

    #[test]
    fn test_false_positive1() {
        const FILTER_SIZE: usize = 10000;
        let mut rng = rand::thread_rng();
        let mut filter = BlockedBloomFilter::create_filter(FILTER_SIZE);
        let rand_keys = rand::seq::index::sample(&mut rng, u32::MAX as usize, FILTER_SIZE * 2);
        for i in 0..FILTER_SIZE {
            filter.add(rand_keys.index(i) as u32);
        }
        for i in 0..FILTER_SIZE {
            assert!(filter.may_contain(rand_keys.index(i) as u32));
        }

        let mut false_pos_count = 0;
        for i in FILTER_SIZE..FILTER_SIZE * 2 {
            if filter.may_contain(rand_keys.index(i) as u32) {
                false_pos_count += 1;
            }
        }
        assert!(
            false_pos_count < 200,
            "false positive rate: {}/10000",
            false_pos_count
        );
    }
}