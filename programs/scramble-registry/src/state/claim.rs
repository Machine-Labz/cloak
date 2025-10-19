use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

/// Claim status enum
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClaimStatus {
    Mined = 0,      // Created but not revealed
    Revealed = 1,   // Revealed within window, ready to consume
    Active = 2,     // Being consumed (alias for Revealed)
    Consumed = 3,   // Fully consumed
    Expired = 4,    // Failed to reveal or consume in time
}

impl ClaimStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ClaimStatus::Mined),
            1 => Some(ClaimStatus::Revealed),
            2 => Some(ClaimStatus::Active),
            3 => Some(ClaimStatus::Consumed),
            4 => Some(ClaimStatus::Expired),
            _ => None,
        }
    }
}

/// Claim - PDA per miner + batch
///
/// Seed: [b"claim", miner_authority, batch_hash, mined_slot_le]
/// Size: 8 + 32 + 32 + 8 + 32 + 16 + 32 + 8 + 8 + 2 + 2 + 8 + 1 + 32 + 32 + 3 = 256 bytes
#[repr(C)]
pub struct Claim {
    /// Discriminator
    pub discriminator: [u8; 8],

    /// Miner authority
    pub miner_authority: Pubkey,

    /// Batch hash
    pub batch_hash: [u8; 32],

    /// Slot when mined
    pub slot: u64,

    /// Slot hash from SlotHashes sysvar
    pub slot_hash: [u8; 32],

    /// Nonce (128-bit)
    pub nonce: u128,

    /// Proof hash (BLAKE3 output)
    pub proof_hash: [u8; 32],

    /// Slot when mined (clock)
    pub mined_at_slot: u64,

    /// Slot when revealed (0 = not revealed)
    pub revealed_at_slot: u64,

    /// How many withdraws consumed this claim
    pub consumed_count: u16,

    /// Batch size k (≤ max_k)
    pub max_consumes: u16,

    /// Expiry slot (revealed_at + claim_window)
    pub expires_at_slot: u64,

    /// Status
    pub status: u8, // ClaimStatus as u8

    /// Reserved for future use
    pub _reserved1: [u8; 32],

    /// Reserved for future use
    pub _reserved2: [u8; 32],

    /// Padding to 256 bytes
    pub _padding: [u8; 3],
}

impl Claim {
    pub const LEN: usize = 256;
    pub const DISCRIMINATOR: [u8; 8] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    pub const SEED: &'static [u8] = b"claim";

    /// Load claim from account
    pub fn from_account(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { &mut *account.borrow_mut_data_unchecked().as_mut_ptr().cast::<Self>() };

        if data.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data)
    }

    /// Create new claim (mined)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        miner_authority: Pubkey,
        batch_hash: [u8; 32],
        slot: u64,
        slot_hash: [u8; 32],
        nonce: u128,
        proof_hash: [u8; 32],
        max_consumes: u16,
        current_slot: u64,
    ) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            miner_authority,
            batch_hash,
            slot,
            slot_hash,
            nonce,
            proof_hash,
            mined_at_slot: current_slot,
            revealed_at_slot: 0,
            consumed_count: 0,
            max_consumes,
            expires_at_slot: 0,
            status: ClaimStatus::Mined as u8,
            _reserved1: [0; 32],
            _reserved2: [0; 32],
            _padding: [0; 3],
        }
    }

    /// Get status enum
    pub fn get_status(&self) -> ClaimStatus {
        ClaimStatus::from_u8(self.status).unwrap_or(ClaimStatus::Expired)
    }

    /// Set status
    pub fn set_status(&mut self, status: ClaimStatus) {
        self.status = status as u8;
    }

    /// Mark as revealed
    pub fn reveal(&mut self, current_slot: u64, claim_window: u64) {
        self.revealed_at_slot = current_slot;
        self.expires_at_slot = current_slot.saturating_add(claim_window);
        self.set_status(ClaimStatus::Revealed);
    }

    /// Increment consumed count
    pub fn consume(&mut self) -> Result<(), ProgramError> {
        if self.consumed_count >= self.max_consumes {
            return Err(ProgramError::InvalidAccountData);
        }

        self.consumed_count = self.consumed_count.saturating_add(1);

        if self.consumed_count == self.max_consumes {
            self.set_status(ClaimStatus::Consumed);
        }

        Ok(())
    }

    /// Check if expired
    pub fn is_expired(&self, current_slot: u64) -> bool {
        if self.expires_at_slot == 0 {
            return false; // Not yet revealed, use reveal window check separately
        }
        current_slot > self.expires_at_slot
    }

    /// Check if revealed
    pub fn is_revealed(&self) -> bool {
        matches!(
            self.get_status(),
            ClaimStatus::Revealed | ClaimStatus::Active
        )
    }

    /// Check if consumable
    pub fn is_consumable(&self, current_slot: u64) -> bool {
        self.is_revealed() && !self.is_expired(current_slot) && self.consumed_count < self.max_consumes
    }
}
