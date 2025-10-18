#![allow(unsafe_op_in_unsafe_fn)]
use pinocchio::{
    account_info::AccountInfo, log::sol_log, program_error::ProgramError, ProgramResult,
};
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

use crate::{
    constants::{
        PROOF_LEN, PROOF_OFF, PUB_AMOUNT_OFF, PUB_LEN, PUB_NF_OFF, PUB_OFF, PUB_OUT_HASH_OFF,
        PUB_ROOT_OFF, RECIP_AMT_OFF, RECIP_OFF, SP1_PUB_LEN, WITHDRAW_VKEY_HASH,
    },
    error::ShieldPoolError,
    state::{Context, NullifierShard, RootsRing},
};

#[derive(Debug)]
pub struct WithdrawAccounts<'info> {
    pool: &'info AccountInfo,
    treasury: &'info AccountInfo,
    roots_ring: &'info AccountInfo,
    nullifier_shard: &'info AccountInfo,
    recipient: &'info AccountInfo,
    _system_program: &'info AccountInfo,
}

impl<'info> WithdrawAccounts<'info> {
    #[inline(always)]
    fn try_from_with_program_id(
        accounts: &'info [AccountInfo],
        program_id: &pinocchio::pubkey::Pubkey,
    ) -> Result<Self, ProgramError> {
        let [pool, treasury, roots_ring, nullifier_shard, recipient, _system_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Pool validation - use runtime program_id instead of hardcoded ID
        if pool.owner() != program_id {
            sol_log("Pool must be owned by program");
            return Err(ShieldPoolError::PoolOwnerNotProgramId.into());
        }
        if !pool.is_writable() {
            sol_log("Pool must be writable");
            return Err(ShieldPoolError::PoolNotWritable.into());
        }

        // Treasury validation
        if !treasury.is_writable() {
            sol_log("Treasury must be writable");
            return Err(ShieldPoolError::TreasuryNotWritable.into());
        }

        // Recipient validation
        if !recipient.is_writable() {
            sol_log("Recipient must be writable");
            return Err(ShieldPoolError::RecipientNotWritable.into());
        }

        Ok(Self {
            pool,
            treasury,
            roots_ring,
            nullifier_shard,
            recipient,
            _system_program,
        })
    }
}

pub struct Withdraw<'info> {
    accounts: WithdrawAccounts<'info>,
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
    recipient_addr: [u8; 32],
    recipient_amount: u64,
}

impl<'info> TryFrom<Context<'info>> for Withdraw<'info> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from_with_program_id(ctx.accounts, ctx.program_id)?;

        // Instruction data validation
        // Format: proof(260) + public_inputs(104) + nullifier(32) + num_outputs(1) + recipient(32) + amount(8)
        // Total: 437 bytes minimum
        if ctx.instruction_data.len() < 437 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        // Extract proof (260 bytes)
        let proof = ctx.instruction_data[PROOF_OFF..(PROOF_OFF + PROOF_LEN)].to_vec();

        // Extract public inputs (104 bytes)
        let public_inputs = ctx.instruction_data[PUB_OFF..(PUB_OFF + PUB_LEN)].to_vec();

        // SAFETY: We validated length above (437 bytes minimum)
        // Extract recipient address and amount using direct pointer reads
        let recipient_addr =
            unsafe { *((ctx.instruction_data.as_ptr().add(RECIP_OFF)) as *const [u8; 32]) };
        let recipient_amount =
            unsafe { *((ctx.instruction_data.as_ptr().add(RECIP_AMT_OFF)) as *const u64) };

        Ok(Self {
            accounts,
            proof,
            public_inputs,
            recipient_addr,
            recipient_amount,
        })
    }
}

impl<'info> Withdraw<'info> {
    #[inline(always)]
    pub fn execute(&self) -> ProgramResult {
        sol_log("Withdraw invoked");

        // Verify SP1 proof
        let full_public_inputs = &self.public_inputs[..SP1_PUB_LEN];
        verify_proof(
            &self.proof,
            full_public_inputs,
            WITHDRAW_VKEY_HASH,
            GROTH16_VK_5_0_0_BYTES,
        )
        .map_err(|_| ShieldPoolError::ProofInvalid)?;

        // SAFETY: We validated public_inputs length in TryFrom (104 bytes)
        // Extract public inputs using direct pointer reads for performance
        let (public_amount, total_fee) = unsafe {
            let public_amount =
                *((self.public_inputs.as_ptr().add(PUB_AMOUNT_OFF - PUB_OFF)) as *const u64);
            let root =
                *((self.public_inputs.as_ptr().add(PUB_ROOT_OFF - PUB_OFF)) as *const [u8; 32]);
            let nf = *((self.public_inputs.as_ptr().add(PUB_NF_OFF - PUB_OFF)) as *const [u8; 32]);
            let outputs_hash_public =
                *((self.public_inputs.as_ptr().add(PUB_OUT_HASH_OFF - PUB_OFF)) as *const [u8; 32]);

            // Verify root exists in RootsRing
            let roots_ring = RootsRing::from_account_info(self.accounts.roots_ring)?;
            if !roots_ring.contains_root(&root) {
                return Err(ShieldPoolError::RootNotFound.into());
            }

            // Check for double-spend
            let mut shard = NullifierShard::from_account_info(self.accounts.nullifier_shard)?;
            if shard.contains_nullifier(&nf) {
                return Err(ShieldPoolError::DoubleSpend.into());
            }

            // Bind outputs_hash to actual recipient and amount
            let mut buf = [0u8; 32 + 8];
            buf[..32].copy_from_slice(&self.recipient_addr);
            buf[32..40].copy_from_slice(&self.recipient_amount.to_le_bytes());
            let outputs_hash_local = *blake3::hash(&buf).as_bytes();

            if outputs_hash_local != outputs_hash_public {
                return Err(ShieldPoolError::InvalidOutputsHash.into());
            }

            // Validate amounts and calculate fee
            if self.recipient_amount > public_amount {
                return Err(ShieldPoolError::InvalidAmount.into());
            }

            let expected_fee = {
                const FIXED: u64 = 2_500_000; // 0.0025 SOL
                const VAR_NUM: u64 = 5; // 0.5%
                const VAR_DEN: u64 = 1_000; // 0.5% = 5/1000
                FIXED + ((public_amount.saturating_mul(VAR_NUM)) / VAR_DEN)
            };
            let total_fee = public_amount - self.recipient_amount;
            if total_fee != expected_fee {
                return Err(ShieldPoolError::Conservation.into());
            }

            // Check pool has sufficient balance
            if self.accounts.pool.lamports() < public_amount {
                return Err(ShieldPoolError::InsufficientLamports.into());
            }

            // Record nullifier before moving funds (fail-closed)
            shard.add_nullifier(&nf)?;

            (public_amount, total_fee)
        };

        // Perform lamport transfers
        unsafe {
            *self.accounts.pool.borrow_mut_lamports_unchecked() -= public_amount;
            *self.accounts.recipient.borrow_mut_lamports_unchecked() += self.recipient_amount;
            *self.accounts.treasury.borrow_mut_lamports_unchecked() += total_fee;
        }

        Ok(())
    }
}
