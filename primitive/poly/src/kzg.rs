mod key;
mod proof;

use crate::commitment::Commitment;
use crate::msm::msm_curve_addtion;
use crate::poly::Coefficients;
use crate::util;
use crate::witness::Witness;
pub use key::EvaluationKey;
pub use proof::Proof;

use parity_scale_codec::{Decode, Encode};
use zkstd::common::*;

/// Kate polynomial commitment params used for prover polynomial domain and proof verification
#[derive(Clone, Debug, PartialEq, Decode, Encode, Default)]
#[allow(dead_code)]
pub struct KzgParams<P: Pairing> {
    pub(crate) g1: Vec<P::G1Affine>,
    pub(crate) g2: P::G2Affine,
    pub(crate) beta_h: P::G2Affine,
}

impl<P: Pairing> KzgParams<P> {
    const ADDED_BLINDING_DEGREE: usize = 6;

    // setup polynomial evaluation domain
    pub fn setup(k: u64, r: P::ScalarField) -> Self {
        // G1, r * G1, r^2 * G1, ..., r^n-1 * G1
        let g1 = (0..=((1 << k) + Self::ADDED_BLINDING_DEGREE as u64))
            .map(|i| {
                let tw = P::G1Projective::ADDITIVE_GENERATOR * r.pow(i);
                P::G1Affine::from(tw)
            })
            .collect::<Vec<_>>();
        let g2 = P::G2Affine::from(P::G2Projective::ADDITIVE_GENERATOR * r);

        Self {
            g1,
            g2,
            beta_h: P::G2Affine::from(P::G2Projective::from(g2) * r),
        }
    }

    // commit polynomial to g1 projective group
    pub fn commit(
        &self,
        poly: &Coefficients<P::ScalarField>,
    ) -> Result<Commitment<P::G1Affine>, Error> {
        self.check_commit_degree_is_within_bounds(poly.degree())?;

        Ok(Commitment::new(msm_curve_addtion::<P>(&self.g1, &poly.0)))
    }

    fn check_commit_degree_is_within_bounds(&self, poly_degree: usize) -> Result<(), Error> {
        match (poly_degree == 0, poly_degree > self.max_degree()) {
            (true, _) => Err(Error::CoefficientsDegreeIsZero),
            (false, true) => Err(Error::CoefficientsDegreeTooLarge),
            (false, false) => Ok(()),
        }
    }

    pub fn verification_key(&self) -> EvaluationKey<P> {
        EvaluationKey::new(self.g1[0], self.g2, self.beta_h)
    }

    pub fn max_degree(&self) -> usize {
        self.g1.len() - 1
    }

    pub fn trim(&self, mut truncated_degree: usize) -> Self {
        truncated_degree += Self::ADDED_BLINDING_DEGREE;
        assert_ne!(truncated_degree, 0);
        assert!(truncated_degree <= self.max_degree());
        if truncated_degree == 1 {
            truncated_degree += 1
        };

        let g1_trunc = self.g1[..=truncated_degree].to_vec();

        Self {
            g1: g1_trunc,
            g2: self.g2,
            beta_h: self.beta_h,
        }
    }

    // create witness for f(a)
    pub fn create_witness(
        &self,
        poly: &Coefficients<P::ScalarField>,
        at: P::ScalarField,
    ) -> Witness<P> {
        // p(x) - p(at) / x - at
        let quotient = poly.divide(&at);

        // p(s)
        let s_eval = self.commit(poly).unwrap();
        // p(at)
        let a_eval = P::G1Projective::ADDITIVE_GENERATOR * poly.evaluate(&at);
        // p(s) - p(at) / s - at
        let q_eval = self.commit(&quotient).unwrap();
        // s - at
        let denominator = P::G2Affine::from(
            P::G2Projective::from(self.g2) - P::G2Projective::ADDITIVE_GENERATOR * at,
        );

        Witness {
            c_eval: P::G1Affine::from(P::G1Projective::from(s_eval.0) - a_eval),
            q_eval: q_eval.0,
            denominator: P::G2PairngRepr::from(-denominator),
            h: P::G2PairngRepr::from(P::G2Affine::from(P::G2Projective::ADDITIVE_GENERATOR)),
        }
    }

    /// Computes a single witness for multiple polynomials at the same point,
    /// by taking a random linear combination of the individual
    /// witnesses. We apply the same optimization mentioned in when
    /// computing each witness; removing f(z).
    pub fn compute_aggregate_witness(
        &self,
        polynomials: &[Coefficients<P::ScalarField>],
        point: &P::ScalarField,
        v_challenge: &P::ScalarField,
    ) -> Coefficients<P::ScalarField> {
        let powers = util::powers_of(v_challenge, polynomials.len() - 1);

        assert_eq!(powers.len(), polynomials.len());

        let numerator: Coefficients<P::ScalarField> = polynomials
            .iter()
            .zip(powers.iter())
            .map(|(poly, v_challenge)| poly * v_challenge)
            .sum();

        numerator.divide(point)
    }
}

#[derive(Debug)]
pub enum Error {
    CoefficientsDegreeIsZero,
    CoefficientsDegreeTooLarge,
}
