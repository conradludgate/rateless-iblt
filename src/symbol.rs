use std::ops::{AddAssign, Sub, SubAssign};

use zerocopy::{little_endian, FromBytes, Immutable, IntoBytes};

use crate::{hash, xor_mut};

#[derive(Debug, Default, FromBytes, Immutable, IntoBytes, Clone, Copy)]
#[repr(C)]
pub struct Symbol {
    pub(crate) value: [u8; 32],
    pub(crate) hash: [u8; 32],
    pub(crate) count: little_endian::I64,
}

impl Sub for Symbol {
    type Output = Symbol;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Symbol {
    fn sub_assign(&mut self, rhs: Self) {
        xor_mut(&mut self.value, &rhs.value);
        xor_mut(&mut self.hash, &rhs.hash);
        self.count -= rhs.count;
    }
}

impl AddAssign for Symbol {
    fn add_assign(&mut self, rhs: Self) {
        xor_mut(&mut self.value, &rhs.value);
        xor_mut(&mut self.hash, &rhs.hash);
        self.count += rhs.count;
    }
}

impl Symbol {
    pub(crate) fn is_pure_cell(&self) -> bool {
        hash(self.value) == self.hash
    }
}
