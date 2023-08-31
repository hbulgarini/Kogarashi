use crate::{
    behave::{Curve, CurveExtended},
    common::CurveGroup,
};

use super::Affine;

/// elliptic curve rational points group
/// rational points group behaves as abelian group
pub trait WeierstrassCurve:
    Curve + Into<<Self as CurveGroup>::Extended> + From<<Self as CurveGroup>::Extended>
{
    // b param
    const PARAM_B: Self::Range;
    // 3b param
    const PARAM_3B: Self::Range;
}

/// rational point affine representation
/// affine representation check that a point is infinite by the struct field
pub trait WeierstrassAffine: WeierstrassCurve + Affine {
    type Projective: WeierstrassProjective<Range = Self::Range>;
    fn to_projective(self) -> Self::Projective;
    fn new_projective(x: Self::Range, y: Self::Range, z: Self::Range) -> Self::Projective;
}

/// rational point projective representation
/// projective representation check that a point is infinite by z coordinate
pub trait WeierstrassProjective: WeierstrassCurve + CurveExtended {
    fn new(x: Self::Range, y: Self::Range, z: Self::Range) -> Self;
}
