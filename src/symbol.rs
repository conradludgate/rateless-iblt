use core::ops::{Add, AddAssign, Sub, SubAssign};

use zerocopy::{little_endian, FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::{hash, xor_mut};

#[derive(Debug, Clone, Copy, FromBytes, Immutable, IntoBytes, Unaligned, KnownLayout)]
#[repr(C)]
pub struct Symbol<T> {
    pub(crate) sum: T,
    pub(crate) checksum: [u8; 16],
    pub(crate) count: little_endian::I64,
}

impl<T: FromZeros> Default for Symbol<T> {
    fn default() -> Self {
        Self {
            sum: T::new_zeroed(),
            checksum: Default::default(),
            count: Default::default(),
        }
    }
}

impl<T: FromBytes + IntoBytes + Immutable> Sub for Symbol<T> {
    type Output = Symbol<T>;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= &rhs;
        self
    }
}

impl<T: FromBytes + IntoBytes + Immutable> SubAssign<&Symbol<T>> for Symbol<T> {
    fn sub_assign(&mut self, rhs: &Self) {
        xor_mut(&mut self.sum, &rhs.sum);
        xor_mut(&mut self.checksum, &rhs.checksum);
        self.count
            .set(self.count.get().wrapping_sub(rhs.count.get()));
    }
}

impl<T: FromBytes + IntoBytes + Immutable> Add for Symbol<T> {
    type Output = Symbol<T>;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T: FromBytes + IntoBytes + Immutable> AddAssign for Symbol<T> {
    fn add_assign(&mut self, rhs: Self) {
        xor_mut(&mut self.sum, &rhs.sum);
        xor_mut(&mut self.checksum, &rhs.checksum);
        self.count
            .set(self.count.get().wrapping_add(rhs.count.get()));
    }
}

impl<T: FromBytes + IntoBytes + Immutable> Symbol<T> {
    pub(crate) fn add_entry(&mut self, value: &T, checksum: &[u8; 16]) {
        xor_mut(&mut self.sum, value);
        xor_mut(&mut self.checksum, checksum);
        self.count.set(self.count.get().wrapping_add(1));
    }

    pub(crate) fn copy(&self) -> Self {
        Symbol {
            sum: T::read_from_bytes(self.sum.as_bytes()).unwrap(),
            checksum: self.checksum,
            count: self.count,
        }
    }
}

impl<T: IntoBytes + Immutable> Symbol<T> {
    pub(crate) fn is_pure_cell(&self) -> bool {
        self.count.get().abs() == 1 && hash(self.sum.as_bytes()) == self.checksum
    }

    pub(crate) fn is_empty_cell(&self) -> bool {
        self.count == 0 && self.checksum == [0; 16]
    }
}

impl<T> Symbol<T> {
    pub(crate) fn encode_count(&mut self, i: u64, n: usize) {
        let p = libm::ceil(crate::index::p(i as f64) * (n as f64)) as i64;
        let d = p - self.count.get();
        self.count.set(d);
    }

    pub(crate) fn decode_count(&mut self, i: usize, n: u64) {
        let p = libm::ceil(crate::index::p(i as f64) * (n as f64)) as i64;
        let d = p - self.count.get();
        self.count.set(d);
    }
}
