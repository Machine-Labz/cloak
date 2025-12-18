use solana_sdk::pubkey::Pubkey;

use crate::{state::SwapState, tests::setup};

#[test]
fn test_swap_state_pda_derivation() {
    let (program_id, _) = setup();

    // Test nullifier
    let nullifier = [0x77u8; 32];

    // Derive SwapState PDA
    let (swap_state_pda, bump) =
        Pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier], &program_id);

    println!("Program ID: {}", program_id);
    println!("Nullifier: {:?}", hex::encode(nullifier));
    println!("SwapState PDA: {}", swap_state_pda);
    println!("Bump: {}", bump);

    // Verify PDA can be derived deterministically
    let (swap_state_pda2, bump2) =
        Pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier], &program_id);

    assert_eq!(
        swap_state_pda, swap_state_pda2,
        "PDA derivation should be deterministic"
    );
    assert_eq!(bump, bump2, "Bump should be deterministic");
}

#[test]
fn test_swap_outputs_hash_computation() {
    // Test parameters for swap
    let output_mint = Pubkey::new_from_array([0xAAu8; 32]);
    let recipient_ata = Pubkey::new_from_array([0xBBu8; 32]);
    let min_output_amount = 1_000_000u64; // 1 USDC (6 decimals)
    let public_amount = 3_000_000_000u64; // 3 SOL

    // Compute swap-mode outputs_hash
    // Formula: H(output_mint || recipient_ata || min_output_amount || public_amount)
    let mut hasher = blake3::Hasher::new();
    hasher.update(output_mint.as_ref());
    hasher.update(recipient_ata.as_ref());
    hasher.update(&min_output_amount.to_le_bytes());
    hasher.update(&public_amount.to_le_bytes());
    let outputs_hash = hasher.finalize();

    println!(
        "Swap outputs_hash: {}",
        hex::encode(outputs_hash.as_bytes())
    );

    // Verify hash is 32 bytes
    assert_eq!(
        outputs_hash.as_bytes().len(),
        32,
        "outputs_hash must be 32 bytes"
    );

    // Verify deterministic computation
    let mut hasher2 = blake3::Hasher::new();
    hasher2.update(output_mint.as_ref());
    hasher2.update(recipient_ata.as_ref());
    hasher2.update(&min_output_amount.to_le_bytes());
    hasher2.update(&public_amount.to_le_bytes());
    let outputs_hash2 = hasher2.finalize();

    assert_eq!(
        outputs_hash.as_bytes(),
        outputs_hash2.as_bytes(),
        "outputs_hash computation should be deterministic"
    );
}

#[test]
fn test_withdraw_swap_instruction_data_format() {
    // This test verifies the instruction data format for WithdrawSwap
    // Format: [proof (260)][public_inputs (104)][duplicate_nullifier (32)]
    //         [output_mint (32)][recipient_ata (32)][min_output_amount (8)]
    // Total: 468 bytes

    let proof = vec![0xFFu8; 260];
    let public_inputs = vec![0xEEu8; 104];
    let nullifier = [0x77u8; 32];
    let output_mint = Pubkey::new_from_array([0xAAu8; 32]);
    let recipient_ata = Pubkey::new_from_array([0xBBu8; 32]);
    let min_output_amount = 1_000_000u64;

    // Build instruction data
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&proof);
    instruction_data.extend_from_slice(&public_inputs);
    instruction_data.extend_from_slice(&nullifier);
    instruction_data.extend_from_slice(output_mint.as_ref());
    instruction_data.extend_from_slice(recipient_ata.as_ref());
    instruction_data.extend_from_slice(&min_output_amount.to_le_bytes());

    println!(
        "WithdrawSwap instruction data size: {} bytes",
        instruction_data.len()
    );

    // Verify size
    assert_eq!(
        instruction_data.len(),
        468,
        "WithdrawSwap instruction data must be exactly 468 bytes"
    );

    // Verify we can parse it back
    assert_eq!(&instruction_data[0..260], &proof[..]);
    assert_eq!(&instruction_data[260..364], &public_inputs[..]);
    assert_eq!(&instruction_data[364..396], &nullifier[..]);
    assert_eq!(&instruction_data[396..428], output_mint.as_ref());
    assert_eq!(&instruction_data[428..460], recipient_ata.as_ref());

    let parsed_min_output = u64::from_le_bytes(instruction_data[460..468].try_into().unwrap());
    assert_eq!(parsed_min_output, min_output_amount);
}

#[test]
fn test_swap_state_size() {
    // Verify SwapState has the expected size
    // Layout: [nullifier: 32][sol_amount: 8][output_mint: 32][recipient_ata: 32]
    //         [min_output_amount: 8][created_slot: 8][bump: 1]
    // Total: 121 bytes

    let expected_size = 32 + 8 + 32 + 32 + 8 + 8 + 1;
    assert_eq!(
        SwapState::SIZE,
        121,
        "SwapState SIZE constant should be 121"
    );
    assert_eq!(
        SwapState::SIZE,
        expected_size,
        "SwapState SIZE should match layout"
    );
}

#[test]
fn test_execute_swap_instruction_data_format() {
    // ExecuteSwap instruction data is just the nullifier (32 bytes)
    let nullifier = [0x77u8; 32];

    let instruction_data = nullifier.to_vec();

    println!(
        "ExecuteSwap instruction data size: {} bytes",
        instruction_data.len()
    );

    assert_eq!(
        instruction_data.len(),
        32,
        "ExecuteSwap instruction data must be exactly 32 bytes (nullifier)"
    );

    // Verify we can parse it back
    let parsed_nullifier: [u8; 32] = instruction_data[0..32].try_into().unwrap();
    assert_eq!(parsed_nullifier, nullifier);
}

#[test]
fn test_swap_instruction_discriminants() {
    // Verify instruction discriminants are correct
    use crate::instructions::ShieldPoolInstruction;

    // WithdrawSwap should be 4
    let withdraw_swap = ShieldPoolInstruction::WithdrawSwap as u8;
    assert_eq!(withdraw_swap, 4, "WithdrawSwap discriminant should be 4");

    // ExecuteSwap should be 5
    let execute_swap = ShieldPoolInstruction::ExecuteSwap as u8;
    assert_eq!(execute_swap, 5, "ExecuteSwap discriminant should be 5");

    // Verify we can convert back
    let parsed = ShieldPoolInstruction::try_from(&4u8).unwrap();
    assert_eq!(
        parsed as u8,
        ShieldPoolInstruction::WithdrawSwap as u8,
        "Should parse discriminant 4 as WithdrawSwap"
    );

    let parsed = ShieldPoolInstruction::try_from(&5u8).unwrap();
    assert_eq!(
        parsed as u8,
        ShieldPoolInstruction::ExecuteSwap as u8,
        "Should parse discriminant 5 as ExecuteSwap"
    );
}

#[test]
fn test_multiple_nullifier_pda_derivations() {
    let (program_id, _) = setup();

    // Test that different nullifiers produce different PDAs
    let nullifier1 = [0x11u8; 32];
    let nullifier2 = [0x22u8; 32];
    let nullifier3 = [0x33u8; 32];

    let (pda1, _) =
        Pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier1], &program_id);

    let (pda2, _) =
        Pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier2], &program_id);

    let (pda3, _) =
        Pubkey::find_program_address(&[SwapState::SEED_PREFIX, &nullifier3], &program_id);

    println!("PDA1: {}", pda1);
    println!("PDA2: {}", pda2);
    println!("PDA3: {}", pda3);

    // Verify they're all different
    assert_ne!(
        pda1, pda2,
        "Different nullifiers should produce different PDAs"
    );
    assert_ne!(
        pda2, pda3,
        "Different nullifiers should produce different PDAs"
    );
    assert_ne!(
        pda1, pda3,
        "Different nullifiers should produce different PDAs"
    );
}

#[test]
fn test_swap_seed_prefix() {
    // Verify SwapState seed prefix is correct
    assert_eq!(
        SwapState::SEED_PREFIX,
        b"swap_state",
        "SwapState seed prefix should be 'swap_state'"
    );
}
