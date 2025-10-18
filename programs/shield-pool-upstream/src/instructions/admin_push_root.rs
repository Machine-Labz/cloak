#![allow(unsafe_op_in_unsafe_fn)]
use pinocchio::{
    account_info::AccountInfo, log::sol_log, program_error::ProgramError, ProgramResult,
};

use crate::{
    constants::ADMIN_AUTHORITY,
    error::ShieldPoolError,
    state::{Context, RootsRing},
};

#[derive(Debug)]
pub struct AdminPushRootAccounts<'info> {
    _admin: &'info AccountInfo,
    roots_ring: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for AdminPushRootAccounts<'info> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [admin, roots_ring, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Verify admin is signer and correct authority
        if !admin.is_signer() {
            sol_log("Admin must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if admin.key() != &ADMIN_AUTHORITY {
            sol_log("Invalid admin authority");
            return Err(ShieldPoolError::BadAccounts.into());
        }

        if !roots_ring.is_writable() {
            sol_log("Roots ring must be writable");
            return Err(ShieldPoolError::BadAccounts.into());
        }

        Ok(Self {
            _admin: admin,
            roots_ring,
        })
    }
}

pub struct AdminPushRoot<'info> {
    accounts: AdminPushRootAccounts<'info>,
    root: [u8; 32],
}

impl<'info> TryFrom<Context<'info>> for AdminPushRoot<'info> {
    type Error = ProgramError;
    #[inline(always)]
    fn try_from(ctx: Context<'info>) -> Result<Self, Self::Error> {
        let accounts = AdminPushRootAccounts::try_from(ctx.accounts)?;

        // Parse root from instruction data (32 bytes)
        if ctx.instruction_data.len() < 32 {
            return Err(ShieldPoolError::InvalidInstructionData.into());
        }

        // SAFETY: We validated length above (32 bytes minimum)
        // Use direct pointer read for performance
        let root = unsafe { *(ctx.instruction_data.as_ptr() as *const [u8; 32]) };

        Ok(Self { accounts, root })
    }
}

impl<'info> AdminPushRoot<'info> {
    #[inline(always)]
    pub fn execute(&self) -> ProgramResult {
        sol_log("AdminPushRoot invoked");

        // Load and update RootsRing
        let mut roots_ring = RootsRing::from_account_info(self.accounts.roots_ring)?;
        roots_ring.push_root(&self.root)?;

        Ok(())
    }
}
