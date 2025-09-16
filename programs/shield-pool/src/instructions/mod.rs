pub mod admin_push_root;
pub mod deposit;
pub mod withdraw;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
