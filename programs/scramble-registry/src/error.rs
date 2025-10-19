use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum ScrambleError {
    /// Invalid proof hash (PoW verification failed)
    InvalidProofHash = 0,
    /// Difficulty target not met
    DifficultyNotMet = 1,
    /// Slot hash mismatch with SlotHashes sysvar
    SlotHashMismatch = 2,
    /// Slot hash not found in SlotHashes (too old)
    SlotHashNotFound = 3,
    /// Reveal window expired
    RevealWindowExpired = 4,
    /// Claim window expired
    ClaimWindowExpired = 5,
    /// Claim already revealed
    AlreadyRevealed = 6,
    /// Claim not revealed
    NotRevealed = 7,
    /// Claim fully consumed
    FullyConsumed = 8,
    /// Batch size exceeds max_k
    BatchSizeTooLarge = 9,
    /// Fee share basis points exceeds maximum
    FeeShareTooHigh = 10,
    /// Invalid miner authority
    InvalidMinerAuthority = 11,
    /// Miner not registered
    MinerNotRegistered = 12,
    /// Invalid admin authority
    InvalidAdminAuthority = 13,
    /// Arithmetic overflow
    ArithmeticOverflow = 14,
    /// Invalid difficulty bounds
    InvalidDifficulty = 15,
    /// Invalid SlotHashes sysvar
    InvalidSlotHashesSysvar = 16,
    /// Unauthorized miner (authority mismatch)
    UnauthorizedMiner = 17,
    /// Slot too old (outside SlotHashes range)
    SlotTooOld = 18,
    /// Batch size exceeds max_k
    BatchSizeExceedsMaxK = 19,
    /// Invalid batch size (must be > 0)
    InvalidBatchSize = 20,
    /// Slot not found in SlotHashes
    SlotNotFound = 21,
    /// Invalid claim status for operation
    InvalidClaimStatus = 22,
    /// Claim expired
    ClaimExpired = 23,
    /// Batch hash mismatch (anti-replay)
    BatchHashMismatch = 24,
    /// Invalid instruction discriminator
    InvalidTag = 25,
}

impl From<ScrambleError> for ProgramError {
    fn from(e: ScrambleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
