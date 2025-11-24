/// ExecuteSwapViaOrca instruction - Atomic on-chain swap via Orca Whirlpool CPI
///
/// **Prerequisites**: wSOL must already be in swap_wsol_ata (call PrepareSwapSol first).
///
/// This instruction only performs the Orca swap CPI. It does NOT transfer SOL or wrap it.
/// The PrepareSwapSol instruction must be called first to wrap SOL → wSOL.
///
/// Account layout:
/// 0. swap_state_pda (writable, signer) - PDA that signs for the swap (closed after swap)
/// 1. swap_wsol_ata (writable) - wSOL ATA (must have wSOL tokens already)
/// 2. recipient_ata (writable) - Output token ATA (from SwapState)
/// 3. whirlpool (writable) - Orca Whirlpool pool
/// 4. token_vault_a (writable) - Pool's wSOL vault
/// 5. token_vault_b (writable) - Pool's output token vault
/// 6. tick_array_0 (writable) - Tick array for swap
/// 7. tick_array_1 (writable) - Tick array for swap
/// 8. tick_array_2 (writable) - Tick array for swap
/// 9. oracle (readonly) - Whirlpool oracle PDA
/// 10. token_program (readonly) - SPL Token program
/// 11. whirlpool_program (readonly) - Orca Whirlpool program
/// 12. payer (writable, signer) - Receives rent from closing SwapState
///
/// Instruction data: [amount: 8][other_amount_threshold: 8][sqrt_price_limit: 16][amount_specified_is_input: 1][a_to_b: 1]
/// Total: 34 bytes
use crate::error::ShieldPoolError;
use crate::state::SwapState;
use crate::ID;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

const SWAP_PARAMS_LEN: usize = 8 + 8 + 16 + 1 + 1; // 34 bytes

// Orca Whirlpool program ID
const ORCA_WHIRLPOOL_PROGRAM: [u8; 32] =
    five8_const::decode_32_const("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

// SPL Token program ID
const TOKEN_PROGRAM: [u8; 32] =
    five8_const::decode_32_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub fn process_execute_swap_via_orca(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse swap parameters
    if data.len() < SWAP_PARAMS_LEN {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(
        data.get(0..8)
            .ok_or(ProgramError::InvalidInstructionData)?
            .try_into()
            .unwrap(),
    );
    let other_amount_threshold = u64::from_le_bytes(
        data.get(8..16)
            .ok_or(ProgramError::InvalidInstructionData)?
            .try_into()
            .unwrap(),
    );
    let sqrt_price_limit = u128::from_le_bytes(
        data.get(16..32)
            .ok_or(ProgramError::InvalidInstructionData)?
            .try_into()
            .unwrap(),
    );
    let amount_specified_is_input = *data
        .get(32)
        .ok_or(ProgramError::InvalidInstructionData)?
        == 1;
    let a_to_b = *data
        .get(33)
        .ok_or(ProgramError::InvalidInstructionData)?
        == 1;

    // Parse accounts (12 accounts now, removed system_program and payer)
    let [swap_state_info, swap_wsol_ata_info, recipient_ata_info, whirlpool_info, token_vault_a_info, token_vault_b_info, tick_array_0_info, tick_array_1_info, tick_array_2_info, oracle_info, token_program_info, whirlpool_program_info, payer_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify Token Program
    if token_program_info.key() != &Pubkey::from(TOKEN_PROGRAM) {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Verify Orca Whirlpool Program
    if whirlpool_program_info.key() != &Pubkey::from(ORCA_WHIRLPOOL_PROGRAM) {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Verify writable accounts
    if !swap_wsol_ata_info.is_writable()
        || !recipient_ata_info.is_writable()
        || !whirlpool_info.is_writable()
        || !token_vault_a_info.is_writable()
        || !token_vault_b_info.is_writable()
        || !tick_array_0_info.is_writable()
        || !tick_array_1_info.is_writable()
        || !tick_array_2_info.is_writable()
    {
        return Err(ShieldPoolError::BadAccounts.into());
    }

    // Read SwapState to get nullifier for PDA derivation
    let swap_state = SwapState::from_account_info(swap_state_info)?;
    let nullifier = swap_state.nullifier();

    // Derive and verify SwapState PDA
    let (expected_swap_state_pda, bump) = pinocchio::pubkey::find_program_address(
        &[SwapState::SEED_PREFIX, &nullifier],
        &ID,
    );

    if swap_state_info.key() != &expected_swap_state_pda {
        return Err(ShieldPoolError::InvalidAccountAddress.into());
    }

    // Prepare signer seeds for CPI
    let bump_bytes = [bump];
    let seeds = [
        Seed::from(SwapState::SEED_PREFIX),
        Seed::from(nullifier.as_ref()),
        Seed::from(bump_bytes.as_ref()),
    ];

    // Build Orca Whirlpool swap instruction
    // Discriminator for Orca swap instruction (Anchor)
    let swap_discriminator = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];

    let mut instruction_data = [0u8; 42]; // 8 (discriminator) + 34 (params) = 42 bytes
    instruction_data[0..8].copy_from_slice(&swap_discriminator);
    instruction_data[8..16].copy_from_slice(&amount.to_le_bytes());
    instruction_data[16..24].copy_from_slice(&other_amount_threshold.to_le_bytes());
    instruction_data[24..40].copy_from_slice(&sqrt_price_limit.to_le_bytes());
    instruction_data[40] = if amount_specified_is_input { 1 } else { 0 };
    instruction_data[41] = if a_to_b { 1 } else { 0 };

    // Account metas for Orca swap (order matters!)
    let account_metas = [
        AccountMeta {
            pubkey: token_program_info.key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: swap_state_info.key(),
            is_signer: true,
            is_writable: false,
        }, // SwapState PDA signs
        AccountMeta {
            pubkey: whirlpool_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: swap_wsol_ata_info.key(),
            is_signer: false,
            is_writable: true,
        }, // Input (wSOL)
        AccountMeta {
            pubkey: token_vault_a_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: recipient_ata_info.key(),
            is_signer: false,
            is_writable: true,
        }, // Output (goes directly to user!)
        AccountMeta {
            pubkey: token_vault_b_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: tick_array_0_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: tick_array_1_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: tick_array_2_info.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: oracle_info.key(),
            is_signer: false,
            is_writable: false,
        },
    ];

    // Account refs must match account_metas order exactly, plus add whirlpool_program
    let account_refs = [
        token_program_info,
        swap_state_info,
        whirlpool_info,
        swap_wsol_ata_info,
        token_vault_a_info,
        recipient_ata_info,
        token_vault_b_info,
        tick_array_0_info,
        tick_array_1_info,
        tick_array_2_info,
        oracle_info,
        whirlpool_program_info, // Orca program for CPI
    ];

    // Execute Orca swap CPI - Output goes directly to user!
    invoke_signed(
        &Instruction {
            program_id: whirlpool_program_info.key(),
            accounts: &account_metas,
            data: &instruction_data,
        },
        &account_refs,
        &[Signer::from(&seeds)],
    )?;

    // Step 2: Close SwapState PDA and return rent to payer
    // This prevents retries and recovers the rent-exempt lamports
    let swap_state_lamports = swap_state_info.lamports();
    let payer_lamports = payer_info.lamports();
    
    // Transfer all lamports from SwapState to payer (closing the account)
    unsafe {
        *swap_state_info.borrow_mut_lamports_unchecked() = 0;
        *payer_info.borrow_mut_lamports_unchecked() = payer_lamports + swap_state_lamports;
    }
    
    // Clear the SwapState data to mark it as closed
    let mut swap_state_data = swap_state_info.try_borrow_mut_data()?;
    swap_state_data.fill(0);

    Ok(())
}
