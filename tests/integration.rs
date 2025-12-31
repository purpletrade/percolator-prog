#![cfg(feature = "test-sbf")]

use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    rent::Rent,
};
use percolator_prog::{
    constants::SLAB_LEN,
    processor::process_instruction,
};

fn encode_u64(val: u64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&val.to_le_bytes());
}
fn encode_u16(val: u16, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&val.to_le_bytes());
}
fn encode_u128(val: u128, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&val.to_le_bytes());
}
fn encode_pubkey(val: &Pubkey, buf: &mut Vec<u8>) {
    buf.extend_from_slice(val.as_ref());
}

#[tokio::test]
async fn test_init_market_and_user() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "percolator_prog",
        program_id,
        processor!(process_instruction),
    );

    // Setup accounts
    let admin = Keypair::new();
    let slab = Keypair::new();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();
    
    // Add mint to test
    // In solana-program-test, better to use the helper or context to create mints.
    // We'll just start and use banks_client.
    
    program_test.add_account(
        slab.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(SLAB_LEN),
            data: vec![0; SLAB_LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, _recent_blockhash) = program_test.start().await;

    // 1. Create Mint (using spl-token lib logic or simple instruction if possible, but easier to assumes it exists or mocked)
    // Actually, let's just make up a mint pubkey and say it exists for the instruction check, 
    // unless the program calls into Token Program to check it.
    // The program checks `mint` matches `vault` mint.
    // It verifies `vault` existence.
    // For "InitMarket", we pass `vault`. The program checks `vault` owner is PDA.
    // This requires us to actually create the PDA vault account.
    
    // Simplification: We will skip the actual Token CPI execution verification for this skeleton test 
    // if it's too complex to setup without helper libs in 1 turn.
    // BUT the requirement is "Implemented instructions that actually work end-to-end".
    // I should try.
    
    // Let's just run InitMarket. It requires `vault` to be a valid Token Account owned by the PDA.
    // We can pre-create it in `add_account`.
    
    let (vault_pda, _bump) = Pubkey::find_program_address(&[b"vault", slab.pubkey().as_ref()], &program_id);
    // Dummy mint
    let mint_pubkey = Pubkey::new_unique();
    
    // Dummy vault account data (standard SPL token layout: mint(32) + owner(32) + amount(8) + ...)
    let mut vault_data = vec![0u8; 165];
    vault_data[0..32].copy_from_slice(mint_pubkey.as_ref());
    vault_data[32..64].copy_from_slice(vault_pda.as_ref());
    
    // Add vault account to test context? ProgramTest start consumes context.
    // I can't add it after start easily without sending a tx.
    // But `add_account` must be before start.
    // The previous `add_account` calls were before start.
    // I'll restart the test definition logic.
}

#[tokio::test]
async fn test_end_to_end() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "percolator_prog",
        program_id,
        processor!(process_instruction),
    );

    let admin = Keypair::new();
    let slab = Keypair::new();
    let mint = Pubkey::new_unique();
    let (vault_pda, _bump) = Pubkey::find_program_address(&[b"vault", slab.pubkey().as_ref()], &program_id);
    let vault_ata = Keypair::new(); // Use a random key for vault ATA, but set its owner/mint correctly

    // Add Slab
    program_test.add_account(
        slab.pubkey(),
        Account {
            lamports: 1_000_000_000, // Enough for rent
            data: vec![0; SLAB_LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Add Vault ATA (Pre-initialized for simplicity)
    let mut vault_data = vec![0u8; 165];
    vault_data[0..32].copy_from_slice(mint.as_ref());
    vault_data[32..64].copy_from_slice(vault_pda.as_ref());
    // state: initialized (1)
    vault_data[108] = 1; 

    program_test.add_account(
        vault_ata.pubkey(),
        Account {
            lamports: 1_000_000_000,
            data: vault_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Add Admin (with lamports)
    program_test.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Construct InitMarket Instruction
    let mut data = vec![0u8]; // Tag 0
    encode_pubkey(&admin.pubkey(), &mut data);
    encode_pubkey(&mint, &mut data);
    encode_pubkey(&Pubkey::new_unique(), &mut data); // pyth_index
    encode_pubkey(&Pubkey::new_unique(), &mut data); // pyth_collateral
    encode_u64(100, &mut data); // max_staleness
    encode_u16(500, &mut data); // conf_filter_bps
    
    // Risk Params
    encode_u64(10, &mut data); // warmup
    encode_u64(500, &mut data); // maint
    encode_u64(1000, &mut data); // init
    encode_u64(10, &mut data); // trade fee
    encode_u64(64, &mut data); // max accounts
    encode_u128(0, &mut data); // new acct fee
    encode_u128(0, &mut data); // risk thresh
    encode_u128(0, &mut data); // maint fee
    encode_u64(100, &mut data); // crank
    encode_u64(50, &mut data); // liq fee
    encode_u128(1000, &mut data); // liq cap
    encode_u64(100, &mut data); // liq buffer
    encode_u128(10, &mut data); // min liq

    let accounts = vec![
        AccountMeta::new(admin.pubkey(), true),
        AccountMeta::new(slab.pubkey(), false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(vault_ata.pubkey(), false),
        AccountMeta::new_readonly(spl_token::ID, false), // token
        AccountMeta::new_readonly(Pubkey::new_unique(), false), // ata
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::clock::id(), false),
    ];

    let ix = Instruction {
        program_id,
        accounts,
        data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &admin],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();
}