pub mod admin_push_root;
pub mod deposit;
pub mod withdraw;

pub use admin_push_root::AdminPushRoot;
pub use deposit::Deposit;
pub use withdraw::Withdraw;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

/// Instruction discriminators for Shield Pool
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShieldPoolInstruction {
    Deposit = 0,
    AdminPushRoot = 1,
    Withdraw = 2,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::AdminPushRoot),
            2 => Ok(Self::Withdraw),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}

impl From<ShieldPoolInstruction> for u8 {
    fn from(instruction: ShieldPoolInstruction) -> u8 {
        instruction as u8
    }
}
