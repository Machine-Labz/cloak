pub mod admin_push_root;
pub mod deposit;
pub mod execute_swap;
pub mod execute_swap_via_orca;
pub mod initialize;
pub mod release_swap_funds;
pub mod withdraw;
pub mod withdraw_swap;

use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

pub enum ShieldPoolInstruction {
    Deposit = 0,
    AdminPushRoot = 1,
    Withdraw = 2,
    Initialize = 3,
    WithdrawSwap = 4,
    ExecuteSwap = 5,
    ReleaseSwapFunds = 6,
    ExecuteSwapViaOrca = 7,
}

impl TryFrom<&u8> for ShieldPoolInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::AdminPushRoot),
            2 => Ok(Self::Withdraw),
            3 => Ok(Self::Initialize),
            4 => Ok(Self::WithdrawSwap),
            5 => Ok(Self::ExecuteSwap),
            6 => Ok(Self::ReleaseSwapFunds),
            7 => Ok(Self::ExecuteSwapViaOrca),
            _ => Err(ShieldPoolError::InvalidTag.into()),
        }
    }
}
