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
    CommitmentAlreadyExists = 0x1035,
    CommitmentLogFull = 0x1036,

    // Math errors
    MathOverflow = 0x1040,
    DivisionByZero = 0x1041,

    // Account errors
    BadAccounts = 0x1050,
    PoolOwnerNotProgramId = 0x1051,
    TreasuryOwnerNotProgramId = 0x1052,
    RootsRingOwnerNotProgramId = 0x1053,
    NullifierShardOwnerNotProgramId = 0x1054,
    PoolNotWritable = 0x1055,
    TreasuryNotWritable = 0x1056,
    RecipientNotWritable = 0x1057,
    InsufficientLamports = 0x1058,
    InvalidAccountOwner = 0x1059,
    InvalidAccountSize = 0x105A,
    CommitmentsNotWritable = 0x105B,

    // Instruction errors
    BadIxLength = 0x1060,
    InvalidInstructionData = 0x1061,
    MissingAccounts = 0x1062,
    InvalidTag = 0x1063,

    // PoW/Scrambler errors
    InvalidMinerAccount = 0x1064,
    InvalidClaimAccount = 0x1065,
    ConsumClaimFailed = 0x1066,

    // Groth16 verifier errors
    InvalidG1Length = 0x1070,
    InvalidG2Length = 0x1071,
    InvalidPublicInputsLength = 0x1072,
    PublicInputGreaterThanFieldSize = 0x1073,
    PreparingInputsG1MulFailed = 0x1074,
    PreparingInputsG1AdditionFailed = 0x1075,
    ProofVerificationFailed = 0x1076,
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
