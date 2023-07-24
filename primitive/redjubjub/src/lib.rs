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

#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

mod constant;
mod hash;
mod keyring;
mod private_key;
mod public_key;
mod signature;

use constant::SIGNING_CTX;
use hash::kogarashi_hash;
pub use private_key::SecretKey;
pub use public_key::{Public, PublicKey};
use signature::Sig as Signature;

use parity_scale_codec::alloc::string::String;
use parity_scale_codec::Encode;
use sp_core::crypto::{CryptoType, DeriveJunction, Pair as TraitPair, SecretStringError};
use sp_std::vec::Vec;
use substrate_bip39::mini_secret_from_entropy;

use bip39::{Language, Mnemonic, MnemonicType};
use rand_core::OsRng;
use zkstd::common::SigUtils;

type Seed = [u8; 32];

fn derive_hard_junction(secret_seed: &Seed, cc: &[u8; 32]) -> Seed {
    ("RedjubjubHDKD", secret_seed, cc).using_encoded(|data| {
        let mut res = [0u8; 32];
        res.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], data).as_bytes());
        res
    })
}

#[derive(Debug)]
pub enum DeriveError {
    /// A soft key was found in the path (and is unsupported).
    SoftKeyInPath,
}

/// A key pair.
#[derive(Clone, Debug)]
pub struct Pair {
    public: PublicKey,
    secret: SecretKey,
}

impl TraitPair for Pair {
    type Public = Public;
    type Seed = Seed;
    type Signature = Signature;
    type DeriveError = DeriveError;

    /// Generate new secure (random) key pair and provide the recovery phrase.
    ///
    /// You can recover the same key later with `from_phrase`.
    fn generate_with_phrase(password: Option<&str>) -> (Pair, String, Seed) {
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        let phrase = mnemonic.phrase();
        let (pair, seed) = Self::from_phrase(phrase, password)
            .expect("All phrases generated by Mnemonic are valid; qed");
        (pair, phrase.to_owned(), seed)
    }

    /// Generate key pair from given recovery phrase and password.
    fn from_phrase(
        phrase: &str,
        password: Option<&str>,
    ) -> Result<(Pair, Seed), SecretStringError> {
        Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|_| SecretStringError::InvalidPhrase)
            .map(|m| Self::from_entropy(m.entropy(), password))
    }

    /// Make a new key pair from secret seed material.
    ///
    /// You should never need to use this; generate(), generate_with_phrase
    fn from_seed(seed: &Seed) -> Pair {
        Self::from_seed_slice(&seed[..]).expect("seed has valid length; qed")
    }

    /// Make a new key pair from secret seed material. The slice must be 32 bytes long or it
    /// will return `None`.
    ///
    /// You should never need to use this; generate(), generate_with_phrase
    fn from_seed_slice(seed_slice: &[u8]) -> Result<Pair, SecretStringError> {
        let secret = SecretKey(kogarashi_hash(seed_slice));
        let public = secret.to_public_key();
        Ok(Self { secret, public })
    }

    /// Derive a child key from a series of given junctions.
    fn derive<Iter: Iterator<Item = DeriveJunction>>(
        &self,
        path: Iter,
        _seed: Option<Seed>,
    ) -> Result<(Pair, Option<Seed>), DeriveError> {
        let mut acc = self.secret.to_bytes();
        for j in path {
            match j {
                DeriveJunction::Soft(_cc) => return Err(DeriveError::SoftKeyInPath),
                DeriveJunction::Hard(cc) => acc = derive_hard_junction(&acc, &cc),
            }
        }
        Ok((Self::from_seed(&acc), Some(acc)))
    }

    /// Get the public key.
    fn public(&self) -> Public {
        Public(self.public.to_bytes())
    }

    /// Sign a message.
    fn sign(&self, message: &[u8]) -> Signature {
        let r = self.secret.sign(message, OsRng);
        Signature(r.to_bytes())
    }

    /// Verify a signature on a message. Returns true if the signature is good.
    fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, pubkey: &Self::Public) -> bool {
        Self::verify_weak(&sig.0[..], message.as_ref(), pubkey)
    }

    /// Verify a signature on a message. Returns true if the signature is good.
    ///
    /// This doesn't use the type system to ensure that `sig` and `pubkey` are the correct
    /// size. Use it only if you're coming from byte buffers and need the speed.
    fn verify_weak<P: AsRef<[u8]>, M: AsRef<[u8]>>(sig: &[u8], message: M, pubkey: P) -> bool {
        let public_key = match PublicKey::from_raw_bytes(pubkey.as_ref()) {
            Some(pk) => pk,
            None => return false,
        };

        let sig = match signature::Signature::from_raw_bytes(sig) {
            Some(s) => s,
            None => return false,
        };

        public_key.validate(message.as_ref(), sig)
    }

    /// Return a vec filled with raw data.
    fn to_raw_vec(&self) -> Vec<u8> {
        self.secret.to_bytes().to_vec()
    }
}

impl Pair {
    /// Make a new key pair from binary data derived from a valid seed phrase.
    ///
    /// This uses a key derivation function to convert the entropy into a seed, then returns
    /// the pair generated from it.
    pub fn from_entropy(entropy: &[u8], password: Option<&str>) -> (Pair, Seed) {
        let seed = mini_secret_from_entropy(entropy, password.unwrap_or(""))
            .expect("32 bytes can always build a key; qed")
            .to_bytes();

        let secret = SecretKey(kogarashi_hash(&seed));
        let public = secret.to_public_key();
        (Pair { secret, public }, secret.to_bytes())
    }

    /// Verify a signature on a message. Returns `true` if the signature is good.
    /// Supports old 0.1.1 deprecated signatures and should be used only for backward
    /// compatibility.
    pub fn verify_deprecated<M: AsRef<[u8]>>(sig: &Signature, message: M, pubkey: &Public) -> bool {
        // Match both schnorrkel 0.1.1 and 0.8.0+ signatures, supporting both wallets
        // that have not been upgraded and those that have.
        match PublicKey::from_bytes(*pubkey.as_ref()) {
            Some(pk) => pk
                .verify_simple_preaudit_deprecated(SIGNING_CTX, message.as_ref(), &sig.0[..])
                .is_ok(),
            None => false,
        }
    }
}

impl CryptoType for Pair {
    type Pair = Pair;
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use rand_core::OsRng;
    use serde_json;
    use sp_core::crypto::{Derive, Ss58Codec, DEV_ADDRESS, DEV_PHRASE};
    use zero_jubjub::Fp;
    use zkstd::behave::Group;

    #[test]
    fn default_phrase_should_be_used() {
        assert_eq!(
            Pair::from_string("//Alice///password", None)
                .unwrap()
                .public(),
            Pair::from_string(&format!("{}//Alice", DEV_PHRASE), Some("password"))
                .unwrap()
                .public(),
        );
        assert_eq!(
            Pair::from_string(&format!("{}/Alice", DEV_PHRASE), None)
                .as_ref()
                .map(Pair::public),
            Pair::from_string("/Alice", None).as_ref().map(Pair::public)
        );
    }

    #[test]
    fn default_address_should_be_used() {
        assert_eq!(
            Public::from_string(&format!("{}/Alice", DEV_ADDRESS)),
            Public::from_string("/Alice")
        );
    }

    #[test]
    fn default_phrase_should_correspond_to_default_address() {
        assert_eq!(
            Pair::from_string(&format!("{}/Alice", DEV_PHRASE), None)
                .unwrap()
                .public(),
            Public::from_string(&format!("{}/Alice", DEV_ADDRESS)).unwrap(),
        );
        assert_eq!(
            Pair::from_string("/Alice", None).unwrap().public(),
            Public::from_string("/Alice").unwrap()
        );
    }

    #[test]
    fn derive_soft_should_work() {
        let pair = Pair::from_seed(&hex!(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ));
        let derive_1 = pair
            .derive(Some(DeriveJunction::soft(1)).into_iter(), None)
            .unwrap()
            .0;
        let derive_1b = pair
            .derive(Some(DeriveJunction::soft(1)).into_iter(), None)
            .unwrap()
            .0;
        let derive_2 = pair
            .derive(Some(DeriveJunction::soft(2)).into_iter(), None)
            .unwrap()
            .0;
        assert_eq!(derive_1.public(), derive_1b.public());
        assert_ne!(derive_1.public(), derive_2.public());
    }

    #[test]
    fn derive_hard_should_work() {
        let pair = Pair::from_seed(&hex!(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ));
        let derive_1 = pair
            .derive(Some(DeriveJunction::hard(1)).into_iter(), None)
            .unwrap()
            .0;
        let derive_1b = pair
            .derive(Some(DeriveJunction::hard(1)).into_iter(), None)
            .unwrap()
            .0;
        let derive_2 = pair
            .derive(Some(DeriveJunction::hard(2)).into_iter(), None)
            .unwrap()
            .0;
        assert_eq!(derive_1.public(), derive_1b.public());
        assert_ne!(derive_1.public(), derive_2.public());
    }

    #[test]
    fn derive_soft_public_should_work() {
        let pair = Pair::from_seed(&hex!(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ));
        let path = Some(DeriveJunction::soft(1));
        let pair_1 = pair.derive(path.clone().into_iter(), None).unwrap().0;
        let public_1 = pair.public().derive(path.into_iter()).unwrap();
        assert_eq!(pair_1.public(), public_1);
    }

    #[test]
    fn derive_hard_public_should_fail() {
        let pair = Pair::from_seed(&hex!(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ));
        let path = Some(DeriveJunction::hard(1));
        assert!(pair.public().derive(path.into_iter()).is_none());
    }

    #[test]
    fn sr_test_vector_should_work() {
        let pair = Pair::from_seed(&hex!(
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        ));
        let public = pair.public();
        assert_eq!(
            public,
            Public::from_raw(hex!(
                "44a996beb1eef7bdcab976ab6d2ca26104834164ecf28fb375600576fcc6eb0f"
            ))
        );
        let message = b"";
        let signature = pair.sign(message);
        assert!(Pair::verify(&signature, &message[..], &public));
    }

    #[test]
    fn generated_pair_should_work() {
        let (pair, _) = Pair::generate();
        let public = pair.public();
        let message = b"Something important";
        let signature = pair.sign(&message[..]);
        assert!(Pair::verify(&signature, &message[..], &public));
    }

    #[test]
    fn messed_signature_should_not_work() {
        let (pair, _) = Pair::generate();
        let public = pair.public();
        let message = b"Signed payload";
        let Signature(mut bytes) = pair.sign(&message[..]);
        bytes[0] = !bytes[0];
        bytes[2] = !bytes[2];
        let signature = Signature(bytes);
        assert!(!Pair::verify(&signature, &message[..], &public));
    }

    #[test]
    fn messed_message_should_not_work() {
        let (pair, _) = Pair::generate();
        let public = pair.public();
        let message = b"Something important";
        let signature = pair.sign(&message[..]);
        assert!(!Pair::verify(
            &signature,
            &b"Something unimportant",
            &public
        ));
    }

    #[test]
    fn seeded_pair_should_work() {
        let pair = Pair::from_seed(b"12345678901234567890123456789012");
        let public = pair.public();
        assert_eq!(
            public,
            Public::from_raw(hex!(
                "741c08a06f41c596608f6774259bd9043304adfa5d3eea62760bd9be97634d63"
            ))
        );
        let message = hex!("2f8c6129d816cf51c374bc7f08c3e63ed156cf78aefb4a6550d97b87997977ee00000000000000000200d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a4500000000000000");
        let signature = pair.sign(&message[..]);
        assert!(Pair::verify(&signature, &message[..], &public));
    }

    #[test]
    fn ss58check_roundtrip_works() {
        let (pair, _) = Pair::generate();
        let public = pair.public();
        let s = public.to_ss58check();
        let cmp = Public::from_ss58check(&s).unwrap();
        assert_eq!(cmp, public);
    }

    #[test]
    fn verify_from_old_wasm_works() {
        // The values in this test case are compared to the output of `node-test.js` in schnorrkel-js.
        //
        // This is to make sure that the wasm library is compatible.
        let pk = Pair::from_seed(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));
        let public = pk.public();
        let js_signature = Signature::from_raw(hex!(
			"28a854d54903e056f89581c691c1f7d2ff39f8f896c9e9c22475e60902cc2b3547199e0e91fa32902028f2ca2355e8cdd16cfe19ba5e8b658c94aa80f3b81a00"
		));
        assert!(Pair::verify_deprecated(
            &js_signature,
            b"SUBSTRATE",
            &public
        ));
        assert!(!Pair::verify(&js_signature, b"SUBSTRATE", &public));
    }

    #[test]
    fn signature_serialization_works() {
        let pair = Pair::from_seed(b"12345678901234567890123456789012");
        let message = b"Something important";
        let signature = pair.sign(&message[..]);
        let serialized_signature = serde_json::to_string(&signature).unwrap();
        // Signature is 64 bytes, so 128 chars + 2 quote chars
        assert_eq!(serialized_signature.len(), 130);
        let signature = serde_json::from_str(&serialized_signature).unwrap();
        assert!(Pair::verify(&signature, &message[..], &pair.public()));
    }

    #[test]
    fn signature_serialization_doesnt_panic() {
        fn deserialize_signature(text: &str) -> Result<Signature, serde_json::error::Error> {
            Ok(serde_json::from_str(text)?)
        }
        assert!(deserialize_signature("Not valid json.").is_err());
        assert!(deserialize_signature("\"Not an actual signature.\"").is_err());
        // Poorly-sized
        assert!(deserialize_signature("\"abc123\"").is_err());
    }

    #[test]
    fn signature_test() {
        for _ in 0..1000 {
            let msg = b"test";
            let randomness = OsRng;
            let priv_key = SecretKey(Fp::random(OsRng));
            let sig = priv_key.sign(msg, randomness);

            let pub_key = priv_key.to_public_key();

            assert!(pub_key.validate(msg, sig))
        }
    }
}
