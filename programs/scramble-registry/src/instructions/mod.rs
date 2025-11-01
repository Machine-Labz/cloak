pub mod consume_claim;
pub mod initialize;
pub mod mine_claim;
pub mod reveal_claim;

use crate::error::ScrambleError;
use pinocchio::program_error::ProgramError;

pub enum ScrambleRegistryInstruction {
    InitializeRegistry = 0,
    RegisterMiner = 1,
    MineClaim = 2,
    RevealClaim = 3,
    ConsumeClaim = 4,
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
            _ => Err(ScrambleError::InvalidTag.into()),
        }
    }
}
