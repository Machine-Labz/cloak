use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShieldPoolError {
    // Root management errors
    InvalidRoot = 0x1000,
    RootNotFound = 0x1001,
    RootsRingFull = 0x1002,

    // Proof verification errors
    ProofInvalid = 0x1010,
    InvalidProofSize = 0x1011,
    InvalidPublicInputs = 0x1012,
    VKeyMismatch = 0x1013,

    // Nullifier errors
    DoubleSpend = 0x1020,
    NullifierShardFull = 0x1021,
    InvalidNullifier = 0x1022,

    // Transaction validation errors
    OutputsMismatch = 0x1030,
    Conservation = 0x1031,
    InvalidOutputsHash = 0x1032,
    InvalidAmount = 0x1033,
    InvalidRecipient = 0x1034,

    // Math errors
    MathOverflow = 0x1040,
    DivisionByZero = 0x1041,

    // Account errors
    BadAccounts = 0x1050,
    PoolOwnerNotProgramId = 0x1051,
    PoolNotWritable = 0x1052,
    TreasuryNotWritable = 0x1053,
    RecipientNotWritable = 0x1054,
    InsufficientLamports = 0x1055,
    InvalidAccountOwner = 0x1056,
    InvalidAccountSize = 0x1057,

    // Instruction errors
    BadIxLength = 0x1060,
    InvalidInstructionData = 0x1061,
    MissingAccounts = 0x1062,
    InvalidTag = 0x1063,
}

impl From<ShieldPoolError> for ProgramError {
    fn from(e: ShieldPoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<ShieldPoolError> for u32 {
    fn from(e: ShieldPoolError) -> Self {
        e as u32
    }
}
