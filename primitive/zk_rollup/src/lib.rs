#![allow(dead_code)]
#![allow(unused_variables)]
mod main_contract;
mod merkle_tree;
mod operator;
mod proof;
mod verifier_contract;

#[cfg(test)]
mod tests {

    use crate::{
        main_contract::MainContract,
        merkle_tree::{MerkleProof, TreeHash},
        operator::{RollupOperator, Transaction, UserData},
    };

    #[test]
    fn test_deposit_and_transfer_works() {
        let mut contract = MainContract::default();

        // deploy contract

        let user = UserData::default();
        contract.deposit(10);

        // Set BATCH_SIZE to 1
        let mut operator = RollupOperator::default();

        // operator.run();
        // should periodically check deposits on the contract

        assert!(contract.check_balance(MerkleProof::default()) == user.balance());

        let t = Transaction::default();

        // t -> contract (not work)

        operator.execute_transaction(t);

        // Should send a proof to the contract inside and update state on chain

        let new_root = TreeHash::default();

        assert!(contract.rollup_state_root == new_root);
        // check calldata on chain
    }
}
