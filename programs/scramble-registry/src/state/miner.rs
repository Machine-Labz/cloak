use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

/// Miner - PDA per authority (anti-key-grinding)
///
/// Seed: [b"miner", miner_authority]
/// Size: 8 (discriminator) + 32 + 8 + 8 + 8 = 64 bytes
#[repr(C)]
pub struct Miner {
    /// Discriminator
    pub discriminator: [u8; 8],

    /// Miner authority (immutable)
    pub authority: Pubkey,

    /// Total claims mined by this authority
    pub total_mined: u64,

    /// Total claims consumed
    pub total_consumed: u64,

    /// Slot when registered
    pub registered_at_slot: u64,
}

impl Miner {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8;
    pub const DISCRIMINATOR: [u8; 8] = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11];
    pub const SEED: &'static [u8] = b"miner";

    /// Load miner from account
    pub fn from_account(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { &mut *account.borrow_mut_data_unchecked().as_mut_ptr().cast::<Self>() };

        if data.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(data)
    }

    /// Create new miner
    pub fn new(authority: Pubkey, current_slot: u64) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            authority,
            total_mined: 0,
            total_consumed: 0,
            registered_at_slot: current_slot,
        }
    }

    /// Record a successful mine
    pub fn record_mine(&mut self) {
        self.total_mined = self.total_mined.saturating_add(1);
    }

    /// Record a consumed claim
    pub fn record_consume(&mut self) {
        self.total_consumed = self.total_consumed.saturating_add(1);
    }
}
