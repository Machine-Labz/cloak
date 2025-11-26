pub mod consume_claim;
pub mod initialize;
pub mod mine_claim;
pub mod reveal_claim;
pub mod top_up_escrow;

use pinocchio::program_error::ProgramError;

use crate::error::ScrambleError;

pub enum ScrambleRegistryInstruction {
    InitializeRegistry = 0,
    RegisterMiner = 1,
    MineClaim = 2,
    RevealClaim = 3,
    ConsumeClaim = 4,
    TopUpEscrow = 5,
}

impl TryFrom<&u8> for ScrambleRegistryInstruction {
    type Error = ProgramError;

    fn try_from(instruction: &u8) -> Result<Self, ProgramError> {
        match instruction {
            0 => Ok(Self::InitializeRegistry),
            1 => Ok(Self::RegisterMiner),
            2 => Ok(Self::MineClaim),
            3 => Ok(Self::RevealClaim),
            4 => Ok(Self::ConsumeClaim),
            5 => Ok(Self::TopUpEscrow),
            _ => Err(ScrambleError::InvalidTag.into()),
        }
    }
}
