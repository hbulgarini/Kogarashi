use core::iter::{self, Sum};
use core::ops::{Add, Deref, DerefMut, Index, Mul, Sub};
use rand_core::RngCore;
use zkstd::common::{FftField, PrimeField, Vec};

/// polynomial coefficients form expression
/// a_n-1 , a_n-2, ... , a_0
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Coefficients<F: PrimeField>(pub Vec<F>);

/// polynomial points-value form expression
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PointsValue<F: PrimeField>(pub Vec<F>);

pub struct Witness<F> {
    s_eval: F,
    a_eval: F,
    q_eval: F,
    denominator: F,
}

impl<F: PrimeField> Deref for Coefficients<F> {
    type Target = [F];

    fn deref(&self) -> &[F] {
        &self.0
    }
}

impl<F: PrimeField> DerefMut for Coefficients<F> {
    fn deref_mut(&mut self) -> &mut [F] {
        &mut self.0
    }
}

impl<F: PrimeField> Index<usize> for PointsValue<F> {
    type Output = F;

    fn index(&self, index: usize) -> &F {
        &self.0[index]
    }
}

impl<F: FftField> Sum for Coefficients<F> {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let sum: Coefficients<F> = iter.fold(Coefficients::default(), |mut res, val| {
            res = &res + &val;
            res
        });
        sum
    }
}

impl<F: FftField> PointsValue<F> {
    pub fn new(coeffs: Vec<F>) -> Self {
        Self(coeffs)
    }

    pub fn format_degree(mut self) -> Self {
        while self.0.last().map_or(false, |c| c == &F::zero()) {
            self.0.pop();
        }
        self
    }
}

impl<F: FftField> Coefficients<F> {
    pub fn new(coeffs: Vec<F>) -> Self {
        Self(coeffs).format_degree()
    }

    // polynomial evaluation domain
    // r^0, r^1, r^2, ..., r^n
    pub fn setup(k: usize, rng: impl RngCore) -> (F, Vec<F>) {
        let randomness = F::random(rng);
        (
            randomness,
            (0..(1 << k))
                .scan(F::one(), |w, _| {
                    let tw = *w;
                    *w *= randomness;
                    Some(tw)
                })
                .collect::<Vec<_>>(),
        )
    }

    // commit polynomial to domain
    pub fn commit(&self, domain: &Vec<F>) -> F {
        assert!(self.0.len() <= domain.len());
        let diff = domain.len() - self.0.len();

        self.0
            .iter()
            .zip(domain.iter().skip(diff))
            .fold(F::zero(), |acc, (a, b)| acc + *a * *b)
    }

    // evaluate polynomial at
    pub fn evaluate(&self, at: &F) -> F {
        self.0
            .iter()
            .rev()
            .fold(F::zero(), |acc, coeff| acc * at + coeff)
    }

    // no remainder polynomial division with at
    // f(x) - f(at) / x - at
    pub fn divide(&self, at: &F) -> Self {
        let mut coeffs = self
            .0
            .iter()
            .rev()
            .scan(F::zero(), |w, coeff| {
                let tmp = *w + coeff;
                *w = tmp * at;
                Some(tmp)
            })
            .collect::<Vec<_>>();
        coeffs.pop();
        coeffs.reverse();
        Self(coeffs)
    }

    /// σ^n - 1
    pub fn t(n: u64, tau: F) -> F {
        tau.pow(n) - F::one()
    }

    /// if hiding degree = 1: (b2*X^(n+1) + b1*X^n - b2*X - b1) + witnesses
    /// if hiding degree = 2: (b3*X^(n+2) + b2*X^(n+1) + b1*X^n - b3*X^2 - b2*X
    pub fn blind<R: RngCore>(&mut self, hiding_degree: usize, rng: &mut R) {
        for i in 0..hiding_degree + 1 {
            let blinding_scalar = F::random(&mut *rng);
            self.0[i] -= blinding_scalar;
            self.0.push(blinding_scalar);
        }
    }

    pub fn format_degree(mut self) -> Self {
        while self.0.last().map_or(false, |c| c == &F::zero()) {
            self.0.pop();
        }
        self
    }

    /// Returns the degree of the [`Coefficients`].
    pub fn degree(&self) -> usize {
        if self.is_zero() {
            return 0;
        }
        assert!(self.0.last().map_or(false, |coeff| coeff != &F::zero()));
        self.0.len() - 1
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|coeff| coeff == &F::zero())
    }

    // create witness for f(a)
    pub fn create_witness(self, at: &F, s: &F, domain: Vec<F>) -> Witness<F> {
        // p(x) - p(at) / x - at
        let quotient = self.divide(at);
        // p(s)
        let s_eval = self.commit(&domain);
        // p(at)
        let a_eval = self.evaluate(at);
        // p(s) - p(at) / s - at
        let q_eval = quotient.evaluate(s);
        // s - at
        let denominator = *s - *at;

        Witness {
            s_eval,
            a_eval,
            q_eval,
            denominator,
        }
    }
}

impl<F: FftField> Add for Coefficients<F> {
    type Output = Coefficients<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let zero = F::zero();
        let (left, right) = if self.0.len() > rhs.0.len() {
            (self.0.iter(), rhs.0.iter().chain(iter::repeat(&zero)))
        } else {
            (rhs.0.iter(), self.0.iter().chain(iter::repeat(&zero)))
        };
        Self::new(left.zip(right).map(|(a, b)| *a + *b).collect())
    }
}

impl<'a, 'b, F: FftField> Add<&'a Coefficients<F>> for &'b Coefficients<F> {
    type Output = Coefficients<F>;

    fn add(self, rhs: &'a Coefficients<F>) -> Self::Output {
        let zero = F::zero();
        let (left, right) = if self.0.len() > rhs.0.len() {
            (self.0.iter(), rhs.0.iter().chain(iter::repeat(&zero)))
        } else {
            (rhs.0.iter(), self.0.iter().chain(iter::repeat(&zero)))
        };
        Coefficients::new(left.zip(right).map(|(a, b)| *a + *b).collect())
    }
}

impl<F: FftField> Sub for Coefficients<F> {
    type Output = Coefficients<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        let zero = F::zero();
        let (left, right) = if self.0.len() > rhs.0.len() {
            (self.0.iter(), rhs.0.iter().chain(iter::repeat(&zero)))
        } else {
            (rhs.0.iter(), self.0.iter().chain(iter::repeat(&zero)))
        };
        Self::new(left.zip(right).map(|(a, b)| *a - *b).collect())
    }
}

impl<'a, 'b, F: FftField> Mul<&'a F> for &'b Coefficients<F> {
    type Output = Coefficients<F>;

    fn mul(self, scalar: &'a F) -> Coefficients<F> {
        Coefficients::new(self.0.iter().map(|coeff| *coeff * scalar).collect())
    }
}

impl<F: FftField> Witness<F> {
    // verify witness
    pub fn verify_eval(self) -> bool {
        self.q_eval * self.denominator == self.s_eval - self.a_eval
    }
}

#[cfg(test)]
mod tests {
    use super::Coefficients;
    use bls_12_381::Fr;
    use rand_core::OsRng;
    use zkstd::common::{Group, PrimeField};

    fn arb_fr() -> Fr {
        Fr::random(OsRng)
    }

    fn arb_poly(k: u32) -> Coefficients<Fr> {
        Coefficients(
            (0..(1 << k))
                .map(|_| Fr::random(OsRng))
                .collect::<Vec<Fr>>(),
        )
    }

    fn naive_multiply<F: PrimeField>(a: Vec<F>, b: Vec<F>) -> Vec<F> {
        let mut c = vec![F::zero(); a.len() + b.len() - 1];
        a.iter().enumerate().for_each(|(i_a, coeff_a)| {
            b.iter().enumerate().for_each(|(i_b, coeff_b)| {
                c[i_a + i_b] += *coeff_a * *coeff_b;
            })
        });
        c
    }

    #[test]
    fn polynomial_scalar() {
        let poly = arb_poly(10);
        let at = arb_fr();
        let scalared = &poly * &at;
        let test = Coefficients(poly.0.into_iter().map(|coeff| coeff * at).collect());
        assert_eq!(scalared, test);
    }

    #[test]
    fn polynomial_division_test() {
        let at = arb_fr();
        let divisor = arb_poly(10);
        // dividend = divisor * quotient
        let factor_poly = vec![-at, Fr::one()];

        // divisor * (x - at) = dividend
        let poly_a = Coefficients(naive_multiply(divisor.0, factor_poly.clone()));

        // dividend / (x - at) = quotient
        let quotient = poly_a.divide(&at);

        // quotient * (x - at) = divident
        let original = Coefficients(naive_multiply(quotient.0, factor_poly));

        assert_eq!(poly_a.0, original.0);
    }
}
