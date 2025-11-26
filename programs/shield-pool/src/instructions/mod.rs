pub mod admin_push_root;
pub mod deposit;
pub mod initialize;
pub mod withdraw;
pub mod withdraw_miner_decoy;

use pinocchio::program_error::ProgramError;

use crate::error::ShieldPoolError;

pub enum ShieldPoolInstruction {
    Deposit = 0,
    AdminPushRoot = 1,
    Withdraw = 2,
    Initialize = 3,
    WithdrawMinerDecoy = 4,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::AdminPushRoot),
            2 => Ok(Self::Withdraw),
            3 => Ok(Self::Initialize),
            4 => Ok(Self::WithdrawMinerDecoy),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}
