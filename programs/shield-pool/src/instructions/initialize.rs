use pinocchio::{
    account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvars::clock::Clock, ProgramResult,
};

pub fn process_initialize_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!(&format!(
        "initialize_instruction_data:{}",
        hex::encode(data)
    ));

    let [pool, roots_ring, treasury, payer, system_program, clock] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify accounts - only payer needs to be a signer for initialization
    // The payer is responsible for funding the account creation and paying rent
    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // PDAs (pool, roots_ring, treasury) do not need to be signers as they are program-derived
    // System program and clock sysvar are never signers

    // Verify system program
    if system_program.key()
        != &Pubkey::from(five8_const::decode_32_const(
            "11111111111111111111111111111111",
        ))
    {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Verify clock
    if clock.key()
        != &Pubkey::from(five8_const::decode_32_const(
            "SysvarC1ock11111111111111111111111111111111",
        ))
    {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Get current timestamp
    let clock = unsafe { &*(clock.borrow_data_unchecked().as_ptr() as *const Clock) };
    let current_timestamp = clock.unix_timestamp;

    // Create PDA accounts by transferring lamports and setting data
    // This is a simplified approach - in production, accounts would be created differently

    // Transfer lamports to create accounts (they will be owned by the program)
    // Note: This is a simplified approach for testing

    // Initialize Pool account (8 bytes for discriminator + 8 bytes for timestamp + 8 bytes for total_deposits)
    unsafe {
        let pool_data = pool.borrow_mut_data_unchecked();
        let pool_data_ptr = pool_data.as_mut_ptr();

        // Set discriminator (0 for Pool)
        *(pool_data_ptr as *mut u64) = 0u64.to_le();
        // Set timestamp
        *((pool_data_ptr.add(8)) as *mut i64) = current_timestamp;
        // Set total_deposits to 0
        *((pool_data_ptr.add(16)) as *mut u64) = 0u64.to_le();
    }

    // Initialize Roots Ring account (8 bytes for discriminator + 8 bytes for head + 8 bytes for tail + 32 * 32 bytes for roots)
    unsafe {
        let roots_ring_data = roots_ring.borrow_mut_data_unchecked();
        let roots_ring_data_ptr = roots_ring_data.as_mut_ptr();

        // Set discriminator (1 for RootsRing)
        *(roots_ring_data_ptr as *mut u64) = 1u64.to_le();
        // Set head to 0
        *((roots_ring_data_ptr.add(8)) as *mut u64) = 0u64.to_le();
        // Set tail to 0
        *((roots_ring_data_ptr.add(16)) as *mut u64) = 0u64.to_le();
        // Initialize roots array to zeros
        for i in 0..32 {
            let root_ptr = roots_ring_data_ptr.add(24 + i * 32);
            std::ptr::write_bytes(root_ptr, 0, 32);
        }
    }

    // For now, just initialize the accounts without transfer
    // The pool will be funded through deposits

    Ok(())
}
