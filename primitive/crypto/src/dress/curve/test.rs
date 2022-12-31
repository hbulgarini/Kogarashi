#[macro_export]
macro_rules! projective_curve_test {
    ($test_name:ident, $field:ident, $curve:ident, $iter_times:expr) => {
        #[cfg(test)]
        mod tests {
            use super::*;
            use paste::paste;
            use rand_core::OsRng;

            fn arb_point<C: Projective>() -> C {
                C::GENERATOR * C::Scalar::random(OsRng)
            }

            paste! {
                #[test]
                fn [< $test_name _is_on_curve_test >]() {
                    assert!($curve::GENERATOR.is_on_curve());
                    assert!($curve::ADDITIVE_IDENTITY.is_on_curve());
                }
            }

            paste! {
                #[test]
                fn [< $test_name _identity_test >]() {
                    let a = arb_point::<$curve>();
                    // a + (-a) = e
                    let identity = a - a;

                    // a + e = a
                    let a_prime = a + $curve::ADDITIVE_IDENTITY;

                    assert_eq!(identity, $curve::ADDITIVE_IDENTITY);
                    assert_eq!(a_prime, a);
                }
            }

            paste! {
                #[test]
                fn [< $test_name _addition_test >]() {
                    for _ in 0..$iter_times {
                        let a = arb_point::<$curve>();
                        let b = arb_point::<$curve>();
                        let c = arb_point::<$curve>();

                        // a + b + c = c + a + b
                        let ab = a + b;
                        let abc = ab + c;
                        let ca = c + a;
                        let cab = ca + b;

                        // 2 * (a + b) = 2 * a + 2 * b
                        let double_ab = ab.double();
                        let aa = a.double();
                        let bb = b.double();
                        let aabb = aa + bb;

                        assert!(abc.is_on_curve());
                        assert!(cab.is_on_curve());
                        assert!(double_ab.is_on_curve());
                        assert!(aabb.is_on_curve());
                        assert_eq!(abc, cab);
                        assert_eq!(double_ab, aabb);
                    }
                }
            }

            paste! {
                #[test]
                fn [< $test_name _doubling_test >]() {
                    for _ in 0..$iter_times {
                        let a = arb_point::<$curve>();

                        // a + a = a * 8
                        let scalared_a = a * $field::from_u64(8);
                        let aa =a.double();
                        let a_4 = aa.double();
                        let a_8 = a_4.double();

                        assert!(scalared_a.is_on_curve());
                        assert!(a_8.is_on_curve());
                        assert_eq!(scalared_a, a_8);
                    }
                }
            }

            paste! {
                #[test]
                fn [< $test_name _scalar_test >]() {
                    for _ in 0..$iter_times {
                        let g = arb_point::<$curve>();

                        // 8 * G + 16 * G = 24 * G
                        let ag = g * $field::new([8, 0, 0, 0]);
                        let bg = g * $field::new([16, 0, 0, 0]);
                        let agbg = ag + bg;

                        let abg = g * $field::new([24, 0, 0, 0]);

                        assert!(agbg.is_on_curve());
                        assert!(abg.is_on_curve());
                        assert_eq!(agbg, abg);
                    }
                }
            }

            paste! {
                #[test]
                fn [< $test_name _conversion_test >]() {
                    for _ in 0..$iter_times {
                        let a = arb_point::<$curve>();

                        // projective -> affine -> projective
                        let affine = a.to_affine();
                        let projective = $curve::from(affine);

                        assert!(projective.is_on_curve());
                        assert_eq!(a, projective);
                    }
                }
            }
        }
    };
}

pub use projective_curve_test;
