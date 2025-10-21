pub mod admin_push_root;
pub mod deposit;
pub mod initialize;
pub mod withdraw;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

pub enum ShieldPoolInstruction {
    Deposit = 0,
    AdminPushRoot = 1,
    Withdraw = 2,
    Initialize = 3,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::AdminPushRoot),
            2 => Ok(Self::Withdraw),
            3 => Ok(Self::Initialize),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}
