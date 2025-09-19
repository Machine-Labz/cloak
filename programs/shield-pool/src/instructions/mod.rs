pub mod admin_push_root;
pub mod deposit;
pub mod initialize;
pub mod withdraw;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ShieldPoolInstruction {
    Initialize = 0,
    Deposit = 1,
    AdminPushRoot = 2,
    Withdraw = 3,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Deposit),
            2 => Ok(Self::AdminPushRoot),
            3 => Ok(Self::Withdraw),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}
