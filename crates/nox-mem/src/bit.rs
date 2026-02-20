use core::{
    ops::Range,
    ptr,
};

use crate::{
    vec::{Vec32, Vector},
    slice,
};

#[derive(Clone)]
pub struct BitField {
    data: Vec32<u8>,
}

impl BitField {

    pub fn new(bytes: u32) -> Self {
        Self {
            data: Vec32::with_len(bytes, 0),
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe {
            ptr::write_bytes(
                self.data.as_mut_ptr(),
                0,
                self.data.len() as usize,
            );
        }
    }

    #[inline(always)]
    pub fn set_bit(&mut self, bit: u64, value: bool) {
        let div = bit / 8;
        let r = (bit - div * 8) as u8;
        let byte = &mut self.data[div as usize];
        let new = *byte & !(1 << r);
        *byte = new | ((value as u8) << r);
    }

    #[inline(always)]
    pub fn any_bit_set(
        &self,
        range: Range<u64>,
    ) -> bool {
        assert!(range.start <= range.end,
            "range start is {}, while range end is {}",
            range.start, range.end,
        );
        let first_byte = range.start / 8;
        let first_idx = first_byte as usize;
        let first_rem = (range.start - first_byte * 8) as u8;
        let last_byte = range.end / 8;
        let last_idx = last_byte as usize;
        let last_rem = (range.end - last_byte * 8) as u8;
        if first_byte == last_byte {
            self.data[first_idx] &
                ((!0u8 ^ ((1 << first_rem) - 1)) &
                ((1 << last_rem) - 1)) != 0
        } else if self.data[first_idx] & (!0u8 ^ ((1 << first_rem) - 1)) != 0 ||
            self.data[last_idx] & ((1 << last_rem) - 1) != 0 
        {
            true
        } else {
            for i in first_idx + 1..last_idx {
                if self.data[i] != 0 {
                    return true
                }
            }
            false
        }
    }

    #[inline(always)]
    pub fn set_bit_range(
        &mut self,
        range: Range<u64>,
        value: bool,
    ) {
        assert!(range.start <= range.end,
            "range start is {}, while range end is {}",
            range.start, range.end,
        );
        if value {
            let first_byte = range.start / 8;
            let first_idx = first_byte as usize;
            let first_rem = (range.start - first_byte * 8) as u8;
            let last_byte = range.end / 8;
            let last_idx = last_byte as usize;
            let last_rem = (range.end - last_byte * 8) as u8;
            if first_byte == last_byte {
                let byte = &mut self.data[first_idx];
                *byte |=
                    (!0u8 ^ ((1 << first_rem) - 1)) &
                    ((1 << last_rem) - 1);
            } else {
                let byte = &mut self.data[first_idx];
                *byte |= !0u8 ^ ((1 << first_rem) - 1);
                let byte = &mut self.data[last_idx];
                *byte |= (1 << last_rem) - 1;
            }
            unsafe {
                ptr::write_bytes(
                    self.data.as_mut_ptr()
                        .add(first_idx + 1),
                    !0u8,
                    last_idx
                        .saturating_sub(first_idx)
                        .saturating_sub(1),
                );
            }
        } else {
            let first_byte = range.start / 8;
            let first_idx = first_byte as usize;
            let first_rem = (range.start - first_byte * 8) as u8;
            let last_byte = range.end / 8;
            let last_idx = last_byte as usize;
            let last_rem = (range.end - last_byte * 8) as u8;
            if first_byte == last_byte {
                let byte = &mut self.data[first_idx];
                *byte &=
                    !((!0u8 ^ ((1 << first_rem) - 1)) &
                    ((1 << last_rem) - 1));
            } else {
                let byte = &mut self.data[first_idx];
                *byte &= !(!0u8 ^ ((1 << first_rem) - 1));
                let byte = &mut self.data[last_idx];
                *byte &= !((1 << last_rem) - 1);
            }
            unsafe {
                ptr::write_bytes(
                    self.data.as_mut_ptr()
                        .add(first_idx + 1),
                    0u8,
                    last_idx
                        .saturating_sub(first_idx)
                        .saturating_sub(1),
                );
            }
        }
    }
    
    #[inline(always)]
    pub fn get(&self, bit: u64) -> bool {
        let div = bit / 8;
        let r = (bit - div * 8) as u8;
        (self.data[div as usize] & (1 << r)) != 0
    }

    #[inline(always)]
    pub fn get_byte(&self, byte: u32) -> u8 {
        self.data[byte as usize]
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            slice_iter: self.data.iter(),
            byte: 0,
            bit: 8,
        }
    }
}

pub struct Iter<'a> {
    slice_iter: slice::Iter<'a, u8>,
    byte: u8,
    bit: u8,
}

impl<'a> IntoIterator for &'a BitField {

    type Item = bool;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Iterator for Iter<'a> {

    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit == 8 {
            self.bit = 0;
            self.byte = self.slice_iter
                .next()
                .copied()?;
        }
        let item = self.byte & (1 << self.bit) != 0;
        self.bit += 1;
        Some(item)
    }
}
