// Copyright (C) 2022-2023 Invers (JP) INC.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Elliptic Curve Coordinate System
//!
//! - [`JubjubAffine`]
//! - [`JubjubProjective`]
//!
//! ## Overview
//!
//! This coordinate provides the functionalities as following.
//!
//! - Curve addition
//! - Curve doubling
//! - Convert each coordinate system
//!
//! ### Reference
//!
//! We implement coordinate system to refer the following.
//! [Projective coordinates for short Weierstrass curves](https://www.hyperelliptic.org/EFD/g1p/auto-shortw-projective.html)

use crate::fp::Fp;
use serde::{Deserialize, Serialize};
use zero_crypto::arithmetic::bits_256::*;
use zero_crypto::common::*;
use zero_crypto::dress::curve::*;

/// The projective form of coordinate
#[derive(Debug, Clone, Copy, Decode, Encode)]
pub struct JubjubProjective {
    pub(crate) x: Fp,
    pub(crate) y: Fp,
    pub(crate) z: Fp,
}

const GENERATOR_X: Fp = Fp::to_mont_form([
    0x7c24d812779a3316,
    0x72e38f4ebd4070f3,
    0x03b3fe93f505a6f2,
    0xc4c71e5a4102960,
]);

const GENERATOR_Y: Fp = Fp::to_mont_form([
    0xd2047ef3463de4af,
    0x01ca03640d236cbf,
    0xd3033593ae386e92,
    0xaa87a50921b80ec,
]);

const PARAM_A: Fp = Fp::zero();

const PARAM_B: Fp = Fp::to_mont_form([4, 0, 0, 0]);

/// The projective form of coordinate
#[derive(Debug, Clone, Copy, Decode, Encode, Serialize, Deserialize)]
pub struct JubjubAffine {
    x: Fp,
    y: Fp,
    is_infinity: bool,
}

curve_operation!(
    Fp,
    Fp,
    PARAM_A,
    PARAM_B,
    JubjubAffine,
    JubjubProjective,
    GENERATOR_X,
    GENERATOR_Y
);

curve_test!(jubjub, Fp, JubjubAffine, JubjubProjective, 100);
