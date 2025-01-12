use crate::{rpc, utils};

use sp_core::{blake2_256, redjubjub::Pair, Encode, Pair as TPair};
use sp_runtime::{codec::Compact, generic::Era, AccountId32, MultiAddress, MultiSignature};

pub async fn create_balance_transfer_xt(signer: Pair, to: AccountId32, amount: u128) -> String {
    let pallet_index: u8 = 5;
    let method_index: u8 = 0;

    let from = AccountId32::from(*signer.public().as_array_ref());
    let to = MultiAddress::Id::<_, u32>(to);
    let amount = Compact(amount);

    let from_nonce = rpc::get_nonce(&from).await;

    let call = (pallet_index, method_index, to.clone(), amount);
    let extra: Vec<u8> = [
        Era::Immortal.encode(),
        Compact(from_nonce).encode(),
        Compact(0u128).encode(),
    ]
    .concat();

    let runtime_version = rpc::get_runtime_version().await;
    let genesis_hash = rpc::get_genesis_hash().await;

    let additional = (
        runtime_version.spec_version,
        runtime_version.transaction_version,
        genesis_hash,
        genesis_hash,
    );

    let signature = {
        let unsigned_signature_scale = (&call, &extra, &additional).encode();
        if unsigned_signature_scale.len() > 256 {
            signer.sign(&blake2_256(&unsigned_signature_scale)[..])
        } else {
            signer.sign(&unsigned_signature_scale)
        }
    };

    let signature_to_encode = Some((
        MultiAddress::Id::<_, u32>(from),
        MultiSignature::Redjubjub(signature),
        extra,
    ));

    let xt = utils::encode_extrinsic(signature_to_encode, call);
    let xt_hex = format!("0x{}", hex::encode(&xt));

    xt_hex
}
