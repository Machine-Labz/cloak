use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

/// ScrambleRegistry - singleton PDA managing PoW parameters and difficulty
///
/// Seed: [b"scramble_registry"]
/// Size: 8 (discriminator) + 32 + 32 + 8 + 8 + 8 + 2 + 8 + 8 + 2 + 32 + 32 + 8 + 8 = 196 bytes
#[repr(C, packed)]
pub struct ScrambleRegistry {
    /// Discriminator (8 bytes, for Anchor compatibility if needed)
    pub discriminator: [u8; 8],

    /// Admin authority (can adjust parameters)
    pub admin: Pubkey,

    /// Current difficulty target (256-bit LE, H must be < this)
    pub current_difficulty: [u8; 32],

    /// Last slot when difficulty was retargeted
    pub last_retarget_slot: u64,

    /// Solutions observed since last retarget
    pub solutions_observed: u64,

    /// Target: 1 solution per this many slots
    pub target_interval_slots: u64,

    /// Scrambler fee share (basis points, ≤ 5000 = 50%)
    pub fee_share_bps: u16,

    /// Slots to reveal after mining
    pub reveal_window: u64,

    /// Slots to consume claim after reveal
    pub claim_window: u64,

    /// Maximum batch size (DoS limit)
    pub max_k: u16,

    /// Minimum difficulty (floor)
    pub min_difficulty: [u8; 32],

    /// Maximum difficulty (ceiling)
    pub max_difficulty: [u8; 32],

    /// Total claims ever mined
    pub total_claims: u64,

    /// Currently active claims
    pub active_claims: u64,
}

impl ScrambleRegistry {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 2 + 8 + 8 + 2 + 32 + 32 + 8 + 8;
    pub const DISCRIMINATOR: [u8; 8] = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]; // Placeholder
    pub const SEED: &'static [u8] = b"scramble_registry";

    /// Load registry from account
    pub fn from_account(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { &mut *account.borrow_mut_data_unchecked().as_mut_ptr().cast::<Self>() };

        // Verify discriminator
        if data.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data)
    }

    /// Create new registry (initialization)
    pub fn new(
        admin: Pubkey,
        initial_difficulty: [u8; 32],
        min_difficulty: [u8; 32],
        max_difficulty: [u8; 32],
        target_interval_slots: u64,
        fee_share_bps: u16,
        reveal_window: u64,
        claim_window: u64,
        max_k: u16,
    ) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            admin,
            current_difficulty: initial_difficulty,
            last_retarget_slot: 0,
            solutions_observed: 0,
            target_interval_slots,
            fee_share_bps,
            reveal_window,
            claim_window,
            max_k,
            min_difficulty,
            max_difficulty,
            total_claims: 0,
            active_claims: 0,
        }
    }

    /// Increment solution counter
    pub fn record_solution(&mut self) {
        self.solutions_observed = self.solutions_observed.saturating_add(1);
        self.total_claims = self.total_claims.saturating_add(1);
    }

    /// Increment active claims
    pub fn increment_active(&mut self) {
        self.active_claims = self.active_claims.saturating_add(1);
    }

    /// Decrement active claims
    pub fn decrement_active(&mut self) {
        self.active_claims = self.active_claims.saturating_sub(1);
    }
}
