#![allow(unsafe_op_in_unsafe_fn)]
use pinocchio::{
    account_info::AccountInfo, log::sol_log, program_error::ProgramError, ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::{error::ShieldPoolError, state::Context};

#[derive(Debug)]
pub struct DepositAccounts<'info> {
    user: &'info AccountInfo,
    pool: &'info AccountInfo,
    _system_program: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for DepositAccounts<'info> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [user, pool, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Account checks
        if !user.is_signer() {
            sol_log("User must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !pool.is_writable() {
            sol_log("Pool account must be writable");
            return Err(ShieldPoolError::PoolNotWritable.into());
        }

        Ok(Self {
            user,
            pool,
            _system_program,
        })
    }
}

pub struct Deposit<'info> {
    accounts: DepositAccounts<'info>,
    amount: u64,
    _commitment: [u8; 32],
}

impl<'info> TryFrom<Context<'info>> for Deposit<'info> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(ctx.accounts)?;

        // Validate instruction data length
        if ctx.instruction_data.len() < 40 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        // SAFETY: We validated length above (40 bytes minimum)
        // Use direct pointer reads for performance
        let amount = unsafe { *(ctx.instruction_data.as_ptr() as *const u64) };
        let commitment = unsafe { *((ctx.instruction_data.as_ptr().add(8)) as *const [u8; 32]) };

        Ok(Self {
            accounts,
            amount,
            _commitment: commitment,
        })
    }
}

impl<'info> Deposit<'info> {
    #[inline(always)]
    pub fn execute(&self) -> ProgramResult {
        sol_log("Deposit invoked");

        // Check user has sufficient lamports
        if self.accounts.user.lamports() < self.amount {
            return Err(ShieldPoolError::InsufficientLamports.into());
        }

        // Transfer SOL from user to pool
        Transfer {
            from: self.accounts.user,
            to: self.accounts.pool,
            lamports: self.amount,
        }
        .invoke()?;

        Ok(())
    }
}
