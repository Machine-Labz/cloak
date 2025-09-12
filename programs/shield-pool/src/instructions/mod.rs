pub mod admin_push_root;
pub mod deposit;
pub mod withdraw;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ShieldPoolInstruction {
    Deposit = 0x01,
    AdminPushRoot = 0x02,
    Withdraw = 0x03,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0x01 => Ok(Self::Deposit),
            0x02 => Ok(Self::AdminPushRoot),
            0x03 => Ok(Self::Withdraw),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}
