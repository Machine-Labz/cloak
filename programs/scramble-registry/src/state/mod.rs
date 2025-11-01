use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// ClaimStatus - Status of a PoW claim
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClaimStatus {
    Mined = 0,
    Revealed = 1,
    Active = 2,
    Consumed = 3,
    Expired = 4,
}

impl ClaimStatus {
    #[inline(always)]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Mined),
            1 => Some(Self::Revealed),
            2 => Some(Self::Active),
            3 => Some(Self::Consumed),
            4 => Some(Self::Expired),
            _ => None,
        }
    }
}

/// ScrambleRegistry: Singleton PDA managing PoW parameters and difficulty
///
/// Layout: [admin: 32][current_difficulty: 32][last_retarget_slot: 8][solutions_observed: 8]
///         [target_interval_slots: 8][fee_share_bps: 2][reveal_window: 8][claim_window: 8]
///         [max_k: 2][min_difficulty: 32][max_difficulty: 32][total_claims: 8][active_claims: 8]
///         Total: 180 bytes
pub struct ScrambleRegistry(*mut u8);

impl ScrambleRegistry {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 8 + 2 + 8 + 8 + 2 + 32 + 32 + 8 + 8;

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if account_info.data_len() != Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn admin(&self) -> &Pubkey {
        unsafe { &*(self.0 as *const Pubkey) }
    }

    #[inline(always)]
    pub fn current_difficulty(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(32) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn last_retarget_slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(64) as *const u64)) }
    }

    #[inline(always)]
    pub fn solutions_observed(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(72) as *const u64)) }
    }

    #[inline(always)]
    pub fn target_interval_slots(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(80) as *const u64)) }
    }

    #[inline(always)]
    pub fn fee_share_bps(&self) -> u16 {
        unsafe { u16::from_le(*(self.0.add(88) as *const u16)) }
    }

    #[inline(always)]
    pub fn reveal_window(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(90) as *const u64)) }
    }

    #[inline(always)]
    pub fn claim_window(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(98) as *const u64)) }
    }

    #[inline(always)]
    pub fn max_k(&self) -> u16 {
        unsafe { u16::from_le(*(self.0.add(106) as *const u16)) }
    }

    #[inline(always)]
    pub fn min_difficulty(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(108) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn max_difficulty(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(140) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn total_claims(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(172) as *const u64)) }
    }

    #[inline(always)]
    pub fn active_claims(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(180) as *const u64)) }
    }

    #[inline(always)]
    pub fn initialize(
        &mut self,
        admin: &Pubkey,
        initial_difficulty: &[u8; 32],
        min_difficulty: &[u8; 32],
        max_difficulty: &[u8; 32],
        target_interval_slots: u64,
        fee_share_bps: u16,
        reveal_window: u64,
        claim_window: u64,
        max_k: u16,
    ) {
        unsafe {
            // admin
            core::ptr::copy_nonoverlapping(admin.as_ref().as_ptr(), self.0, 32);
            // current_difficulty
            core::ptr::copy_nonoverlapping(initial_difficulty.as_ptr(), self.0.add(32), 32);
            // last_retarget_slot
            *(self.0.add(64) as *mut u64) = 0u64.to_le();
            // solutions_observed
            *(self.0.add(72) as *mut u64) = 0u64.to_le();
            // target_interval_slots
            *(self.0.add(80) as *mut u64) = target_interval_slots.to_le();
            // fee_share_bps
            *(self.0.add(88) as *mut u16) = fee_share_bps.to_le();
            // reveal_window
            *(self.0.add(90) as *mut u64) = reveal_window.to_le();
            // claim_window
            *(self.0.add(98) as *mut u64) = claim_window.to_le();
            // max_k
            *(self.0.add(106) as *mut u16) = max_k.to_le();
            // min_difficulty
            core::ptr::copy_nonoverlapping(min_difficulty.as_ptr(), self.0.add(108), 32);
            // max_difficulty
            core::ptr::copy_nonoverlapping(max_difficulty.as_ptr(), self.0.add(140), 32);
            // total_claims
            *(self.0.add(172) as *mut u64) = 0u64.to_le();
            // active_claims
            *(self.0.add(180) as *mut u64) = 0u64.to_le();
        }
    }

    #[inline(always)]
    pub fn record_solution(&mut self) {
        unsafe {
            let current = self.solutions_observed();
            *(self.0.add(72) as *mut u64) = current.saturating_add(1).to_le();

            let total = self.total_claims();
            *(self.0.add(172) as *mut u64) = total.saturating_add(1).to_le();
        }
    }

    #[inline(always)]
    pub fn increment_active(&mut self) {
        unsafe {
            let active = self.active_claims();
            *(self.0.add(180) as *mut u64) = active.saturating_add(1).to_le();
        }
    }

    #[inline(always)]
    pub fn decrement_active(&mut self) {
        unsafe {
            let active = self.active_claims();
            *(self.0.add(180) as *mut u64) = active.saturating_sub(1).to_le();
        }
    }
}

/// Miner: PDA per authority (anti-key-grinding)
///
/// Layout: [authority: 32][total_mined: 8][total_consumed: 8][registered_at_slot: 8]
/// Total: 56 bytes
pub struct Miner(*mut u8);

impl Miner {
    pub const SIZE: usize = 32 + 8 + 8 + 8;

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if account_info.data_len() != Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn authority(&self) -> &Pubkey {
        unsafe { &*(self.0 as *const Pubkey) }
    }

    #[inline(always)]
    pub fn total_mined(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(32) as *const u64)) }
    }

    #[inline(always)]
    pub fn total_consumed(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(40) as *const u64)) }
    }

    #[inline(always)]
    pub fn registered_at_slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(48) as *const u64)) }
    }

    #[inline(always)]
    pub fn initialize(&mut self, authority: &Pubkey, current_slot: u64) {
        unsafe {
            // authority
            core::ptr::copy_nonoverlapping(authority.as_ref().as_ptr(), self.0, 32);
            // total_mined
            *(self.0.add(32) as *mut u64) = 0u64.to_le();
            // total_consumed
            *(self.0.add(40) as *mut u64) = 0u64.to_le();
            // registered_at_slot
            *(self.0.add(48) as *mut u64) = current_slot.to_le();
        }
    }

    #[inline(always)]
    pub fn record_mine(&mut self) {
        unsafe {
            let mined = self.total_mined();
            *(self.0.add(32) as *mut u64) = mined.saturating_add(1).to_le();
        }
    }

    #[inline(always)]
    pub fn record_consume(&mut self) {
        unsafe {
            let consumed = self.total_consumed();
            *(self.0.add(40) as *mut u64) = consumed.saturating_add(1).to_le();
        }
    }
}

/// Claim: PDA per miner + batch
///
/// Layout: [miner_authority: 32][batch_hash: 32][slot: 8][slot_hash: 32][nonce: 16]
///         [proof_hash: 32][mined_at_slot: 8][revealed_at_slot: 8][consumed_count: 2]
///         [max_consumes: 2][expires_at_slot: 8][status: 1][_reserved: 75]
/// Total: 256 bytes (aligned)
pub struct Claim(*mut u8);

impl Claim {
    pub const SIZE: usize = 256;

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if account_info.data_len() != Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn miner_authority(&self) -> &Pubkey {
        unsafe { &*(self.0 as *const Pubkey) }
    }

    #[inline(always)]
    pub fn batch_hash(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(32) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(64) as *const u64)) }
    }

    #[inline(always)]
    pub fn slot_hash(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(72) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn nonce(&self) -> u128 {
        unsafe { u128::from_le(*(self.0.add(104) as *const u128)) }
    }

    #[inline(always)]
    pub fn proof_hash(&self) -> &[u8; 32] {
        unsafe { &*(self.0.add(120) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn mined_at_slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(152) as *const u64)) }
    }

    #[inline(always)]
    pub fn revealed_at_slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(160) as *const u64)) }
    }

    #[inline(always)]
    pub fn consumed_count(&self) -> u16 {
        unsafe { u16::from_le(*(self.0.add(168) as *const u16)) }
    }

    #[inline(always)]
    pub fn max_consumes(&self) -> u16 {
        unsafe { u16::from_le(*(self.0.add(170) as *const u16)) }
    }

    #[inline(always)]
    pub fn expires_at_slot(&self) -> u64 {
        unsafe { u64::from_le(*(self.0.add(172) as *const u64)) }
    }

    #[inline(always)]
    pub fn status(&self) -> ClaimStatus {
        unsafe { ClaimStatus::from_u8(*self.0.add(180)).unwrap_or(ClaimStatus::Expired) }
    }

    #[inline(always)]
    fn set_status(&mut self, status: ClaimStatus) {
        unsafe {
            *self.0.add(180) = status as u8;
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        miner_authority: &Pubkey,
        batch_hash: &[u8; 32],
        slot: u64,
        slot_hash: &[u8; 32],
        nonce: u128,
        proof_hash: &[u8; 32],
        max_consumes: u16,
        current_slot: u64,
    ) {
        unsafe {
            // miner_authority
            core::ptr::copy_nonoverlapping(miner_authority.as_ref().as_ptr(), self.0, 32);
            // batch_hash
            core::ptr::copy_nonoverlapping(batch_hash.as_ptr(), self.0.add(32), 32);
            // slot
            *(self.0.add(64) as *mut u64) = slot.to_le();
            // slot_hash
            core::ptr::copy_nonoverlapping(slot_hash.as_ptr(), self.0.add(72), 32);
            // nonce
            *(self.0.add(104) as *mut u128) = nonce.to_le();
            // proof_hash
            core::ptr::copy_nonoverlapping(proof_hash.as_ptr(), self.0.add(120), 32);
            // mined_at_slot
            *(self.0.add(152) as *mut u64) = current_slot.to_le();
            // revealed_at_slot
            *(self.0.add(160) as *mut u64) = 0u64.to_le();
            // consumed_count
            *(self.0.add(168) as *mut u16) = 0u16.to_le();
            // max_consumes
            *(self.0.add(170) as *mut u16) = max_consumes.to_le();
            // expires_at_slot
            *(self.0.add(172) as *mut u64) = 0u64.to_le();
            // status
            *self.0.add(180) = ClaimStatus::Mined as u8;
            // Zero out reserved space
            core::ptr::write_bytes(self.0.add(181), 0, 75);
        }
    }

    #[inline(always)]
    pub fn reveal(&mut self, current_slot: u64, claim_window: u64) {
        unsafe {
            *(self.0.add(160) as *mut u64) = current_slot.to_le();
            *(self.0.add(172) as *mut u64) = current_slot.saturating_add(claim_window).to_le();
        }
        self.set_status(ClaimStatus::Revealed);
    }

    #[inline(always)]
    pub fn consume(&mut self) -> Result<(), ProgramError> {
        let consumed = self.consumed_count();
        let max = self.max_consumes();

        if consumed >= max {
            return Err(ProgramError::InvalidAccountData);
        }

        unsafe {
            *(self.0.add(168) as *mut u16) = consumed.saturating_add(1).to_le();
        }

        if consumed + 1 == max {
            self.set_status(ClaimStatus::Consumed);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn is_expired(&self, current_slot: u64) -> bool {
        let expires = self.expires_at_slot();
        if expires == 0 {
            return false;
        }
        current_slot > expires
    }

    #[inline(always)]
    pub fn is_revealed(&self) -> bool {
        matches!(self.status(), ClaimStatus::Revealed | ClaimStatus::Active)
    }

    #[inline(always)]
    pub fn is_consumable(&self, current_slot: u64) -> bool {
        self.is_revealed()
            && !self.is_expired(current_slot)
            && self.consumed_count() < self.max_consumes()
    }

    /// Check if this claim is a wildcard (can be used for any batch)
    /// Wildcard claims have batch_hash = [0; 32]
    #[inline(always)]
    pub fn is_wildcard(&self) -> bool {
        self.batch_hash() == &[0u8; 32]
    }
}
