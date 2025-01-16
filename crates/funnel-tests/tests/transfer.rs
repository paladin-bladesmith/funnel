use std::collections::HashMap;

use solana_sdk::account::Account;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, TransactionError};
use solana_sdk::{system_instruction, system_transaction};
use svm_test::Svm;

#[test]
fn program_owned_payer() {
    let mut svm: Svm<_> = Svm::new(HashMap::<Pubkey, Account>::default());

    // Create a `from` owned by the funnel.
    let from = Keypair::new();
    svm.set(
        from.pubkey(),
        Account {
            lamports: 10u64.pow(9),
            owner: funnel::ID,
            rent_epoch: u64::MAX,
            ..Default::default()
        },
    );

    // Setup `to`.
    let to = Keypair::new();
    svm.set(to.pubkey(), Account { lamports: 10u64.pow(9), ..Default::default() });

    // Sign as `from` to move lamports.
    let tx = system_transaction::transfer(&from, &to.pubkey(), 1, svm.blockhash());
    let failed = svm.execute_transaction(tx).unwrap_err();

    // Fails as non system program owned accounts cannot pay fees.
    assert_eq!(failed.err, TransactionError::InvalidAccountForFee);
}

#[test]
fn program_owned_signer() {
    let mut svm: Svm<_> = Svm::new(HashMap::<Pubkey, Account>::default());

    // Setup a payer.
    let payer = Keypair::new();
    svm.set(payer.pubkey(), Account { lamports: 10u64.pow(9), ..Default::default() });

    // Create a `from` owned by the funnel.
    let from = Keypair::new();
    svm.set(
        from.pubkey(),
        Account {
            lamports: 10u64.pow(9),
            owner: funnel::ID,
            rent_epoch: u64::MAX,
            ..Default::default()
        },
    );

    // Setup `to`.
    let to = Keypair::new();
    svm.set(to.pubkey(), Account { lamports: 10u64.pow(9), ..Default::default() });

    // Sign as `from` to move lamports.
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&from.pubkey(), &to.pubkey(), 1)],
        Some(&payer.pubkey()),
        &[&payer, &from],
        svm.blockhash(),
    );
    let failed = svm.execute_transaction(tx).unwrap_err();

    // Fails as non system program owned accounts cannot pay fees.
    assert_eq!(
        failed.err,
        TransactionError::InstructionError(0, InstructionError::ExternalAccountLamportSpend)
    );
}

#[test]
fn program_owned_signer_memo() {
    let mut svm: Svm<_> = Svm::new(HashMap::<Pubkey, Account>::default());
    svm.load_program(spl_memo::ID, "spl_memo");

    // Setup a payer.
    let payer = Keypair::new();
    svm.set(payer.pubkey(), Account { lamports: 10u64.pow(9), ..Default::default() });

    // Create a `from` owned by the funnel.
    let from = Keypair::new();
    svm.set(
        from.pubkey(),
        Account {
            lamports: 10u64.pow(9),
            owner: funnel::ID,
            rent_epoch: u64::MAX,
            ..Default::default()
        },
    );

    // Sign a memo transaction.
    let tx = Transaction::new_signed_with_payer(
        &[spl_memo::build_memo(b"hello world", &[&from.pubkey()])],
        Some(&payer.pubkey()),
        &[&payer, &from],
        svm.blockhash(),
    );
    svm.execute_transaction(tx).unwrap();
}
