use std::ops::{Add, AddAssign, Sub, SubAssign};

use zerocopy::{little_endian, FromBytes, FromZeros, Immutable, IntoBytes};

use crate::{hash, xor_mut};

#[derive(Debug, FromBytes, Immutable, IntoBytes, Clone, Copy)]
#[repr(C)]
pub struct Symbol<T> {
    pub(crate) sum: T,
    pub(crate) checksum: [u8; 32],
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
        self -= rhs;
        self
    }
}

impl<T: FromBytes + IntoBytes + Immutable> SubAssign for Symbol<T> {
    fn sub_assign(&mut self, rhs: Self) {
        xor_mut(&mut self.sum, &rhs.sum);
        xor_mut(&mut self.checksum, &rhs.checksum);
        self.count -= rhs.count;
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
        self.count += rhs.count;
    }
}

impl<T: IntoBytes + Immutable> Symbol<T> {
    pub(crate) fn is_pure_cell(&self) -> bool {
        self.count.get().abs() == 1 && hash(self.sum.as_bytes()) == self.checksum
    }

    pub(crate) fn is_empty_cell(&self) -> bool {
        self.count.get() == 0
            && self.checksum == [0; 32]
            && self.sum.as_bytes().iter().all(|x| *x == 0)
    }
}
