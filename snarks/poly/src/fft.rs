use sp_std::{vec, vec::Vec};
use zero_jubjub::Fr;

pub struct Fft {
    k: usize,
    n: usize,
    twiddles: Vec<Vec<Fr>>,
    reversed_indexes: Vec<usize>,
}

impl Fft {
    pub fn new(k: usize) -> Self {
        let n = 1 << k;
        let twiddles = Vec::with_capacity(n / 2);
        let mut reversed_indexes = vec![0; n];
        if k % 2 == 1 {
            
        }
        Fft { k, n, twiddles, reversed_indexes }
    }

    pub fn fft(&self, coeffs: &mut [Fr]) {

    }
}

#[cfg(test)]
mod fft_tests {
    use super::*;

    #[test]
    fn fft_test() {
        let mut a_coeffs = vec![];
        let mut b_coeffs = vec![];
        let exponent_of_two = 6;
        let poly_degree = (1u64 << exponent_of_two) as usize;
        let mut naive_result = vec![Fr::zero(); poly_degree * 2];
        for _ in 0..poly_degree {
            let rng = &mut rand::thread_rng();
            a_coeffs.push(Fr::random(rng));
        }
        for _ in 0..poly_degree {
            let rng = &mut rand::thread_rng();
            b_coeffs.push(Fr::random(rng));
        }
        for a in 0..poly_degree {
            for b in 0..poly_degree {
                naive_result[a + b] = a_coeffs[a] * b_coeffs[b];
            }
        }
        assert_eq!(naive_result.len(), poly_degree * 2)
    }
}