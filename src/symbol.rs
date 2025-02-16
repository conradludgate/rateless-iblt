use core::ops::{Add, AddAssign, Sub, SubAssign};

use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes};

use crate::{hash, xor_mut};

#[derive(Debug, Clone, Copy)]
pub struct Symbol<T> {
    pub(crate) sum: T,
    pub(crate) checksum: [u8; 16],
    pub(crate) count: i64,
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
        self.count = self.count.wrapping_sub(rhs.count);
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
        self.count = self.count.wrapping_add(rhs.count);
    }
}

impl<T: FromBytes + IntoBytes + Immutable> Symbol<T> {
    pub(crate) fn add_entry(&mut self, value: &T, checksum: &[u8; 16]) {
        xor_mut(&mut self.sum, value);
        xor_mut(&mut self.checksum, checksum);
        self.count = self.count.wrapping_add(1);
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
        self.count.abs() == 1 && hash(self.sum.as_bytes()) == self.checksum
    }

    pub(crate) fn is_empty_cell(&self) -> bool {
        self.count == 0 && self.checksum == [0; 16]
    }
}
