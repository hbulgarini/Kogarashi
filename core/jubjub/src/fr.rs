use crate::arithmetic::{add, double, mul, sub};
use parity_scale_codec::{Decode, Encode};

pub(crate) const MODULUS: &[u64; 4] = &[
    0xd0970e5ed6f72cb7,
    0xa6682093ccc81082,
    0x06673b0101343b00,
    0x0e7db4ea6533afa9,
];

pub(crate) const INV: u64 = 0x1ba3a358ef788ef9;

#[derive(Debug, Clone, Decode, Encode)]
pub(crate) struct Fr(pub(crate) [u64; 4]);

impl Fr {
    pub fn add_asign(&mut self, other: Self) {
        self.0 = add(&self.0, &other.0);
    }

    pub fn sub_assign(&mut self, other: Self) {
        self.0 = sub(&self.0, &other.0);
    }

    pub fn double_assign(&mut self) {
        self.0 = double(&self.0)
    }

    pub fn mul_assign(&mut self, other: Self) {
        self.0 = mul(&self.0, &other.0)
    }
}