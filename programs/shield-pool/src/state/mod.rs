use crate::{error::ShieldPoolError, ID};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

/// Pool: Stores the token mint for this shield pool
/// Layout: [mint: 32 bytes]
/// If mint == Pubkey::default() (all zeros), pool handles native SOL
/// Otherwise, pool handles the specified SPL token
pub struct Pool(*mut u8);

impl Pool {
    pub const SIZE: usize = 32; // Just the mint pubkey

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if account_info.owner() != &ID {
            return Err(ShieldPoolError::InvalidAccountOwner.into());
        }
        if account_info.data_len() != Self::SIZE {
            return Err(ShieldPoolError::InvalidAccountSize.into());
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn mint(&self) -> Pubkey {
        unsafe {
            let mut mint_bytes = [0u8; 32];
            core::ptr::copy_nonoverlapping(self.0, mint_bytes.as_mut_ptr(), 32);
            Pubkey::from(mint_bytes)
        }
    }

    #[inline(always)]
    pub fn set_mint(&mut self, mint: &Pubkey) {
        unsafe {
            core::ptr::copy_nonoverlapping(mint.as_ref().as_ptr(), self.0, 32);
        }
    }

    #[inline(always)]
    pub fn is_native(&self) -> bool {
        self.mint() == Pubkey::default()
    }
}

/// CommitmentQueue: Fixed-size ring buffer storing recent deposit commitments.
/// Layout:
/// [total_commits: u64][reserved: u64][commitments: CAPACITY * 32 bytes]
pub struct CommitmentQueue(*mut u8);

impl CommitmentQueue {
    pub const HEADER_SIZE: usize = 16; // 8 bytes count + 8 bytes reserved
    pub const CAPACITY: usize = 256;
    pub const SIZE: usize = Self::HEADER_SIZE + Self::CAPACITY * 32; // 16 + 8192 = 8208 bytes

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        if account_info.owner() != &ID {
            return Err(ShieldPoolError::InvalidAccountOwner.into());
        }
        if account_info.data_len() != Self::SIZE {
            return Err(ShieldPoolError::InvalidAccountSize.into());
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn total_commits(&self) -> u64 {
        unsafe { u64::from_le(*(self.0 as *const u64)) }
    }

    #[inline(always)]
    fn set_total_commits(&mut self, value: u64) {
        unsafe {
            *(self.0 as *mut u64) = value.to_le();
        }
    }

    #[inline(always)]
    fn slot_offset(slot: usize) -> usize {
        Self::HEADER_SIZE + slot * 32
    }

    #[inline(always)]
    unsafe fn write_commitment(&mut self, slot: usize, commitment: &[u8; 32]) {
        core::ptr::copy_nonoverlapping(
            commitment.as_ptr(),
            self.0.add(Self::slot_offset(slot)),
            32,
        );
    }

    #[inline(always)]
    unsafe fn read_commitment(&self, slot: usize, out: &mut [u8; 32]) {
        core::ptr::copy_nonoverlapping(self.0.add(Self::slot_offset(slot)), out.as_mut_ptr(), 32);
    }

    #[inline(always)]
    pub fn contains(&self, commitment: &[u8; 32]) -> bool {
        let total = self.total_commits();
        let count = core::cmp::min(total, Self::CAPACITY as u64);
        if count == 0 {
            return false;
        }

        let start_index = total.saturating_sub(count);
        let mut buffer = [0u8; 32];
        for offset in 0..count {
            let index = start_index + offset;
            let slot = (index % Self::CAPACITY as u64) as usize;
            unsafe {
                self.read_commitment(slot, &mut buffer);
            }
            if &buffer == commitment {
                return true;
            }
        }
        false
    }

    #[inline(always)]
    pub fn append(&mut self, commitment: &[u8; 32]) -> Result<u64, ProgramError> {
        let total = self.total_commits();
        if total == u64::MAX {
            return Err(ShieldPoolError::CommitmentLogFull.into());
        }

        let slot = (total % Self::CAPACITY as u64) as usize;
        unsafe {
            self.write_commitment(slot, commitment);
        }
        self.set_total_commits(total + 1);
        Ok(total)
    }
}

/// RootsRing: Fixed-size ring buffer of recent Merkle roots
/// Layout: [head: u8][pad: 7][roots: 64 * 32 bytes] => total = 8 + 2048 = 2056 bytes
pub struct RootsRing(*mut u8);

impl RootsRing {
    pub const SIZE: usize = 8 + 64 * 32; // 2056 bytes
    pub const MAX_ROOTS: usize = 64;

    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<Self, ProgramError> {
        if account_info.data_len() != Self::SIZE {
            return Err(ShieldPoolError::InvalidAccountSize.into());
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn head(&self) -> u8 {
        unsafe { *self.0 }
    }

    #[inline(always)]
    pub fn push_root(&mut self, root: &[u8; 32]) -> Result<(), ProgramError> {
        let current_head = self.head();
        let new_head = (current_head + 1) % (Self::MAX_ROOTS as u8);

        unsafe {
            // Update head
            *self.0 = new_head;

            // Store root at new position
            let root_offset = 8 + (new_head as usize) * 32;
            let root_ptr = self.0.add(root_offset);
            core::ptr::copy_nonoverlapping(root.as_ptr(), root_ptr, 32);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn contains_root(&self, target_root: &[u8; 32]) -> bool {
        unsafe {
            // Unrolled first 8 comparisons for common cases
            for i in 0..8 {
                let root_offset = 8 + i * 32;
                let root_ptr = self.0.add(root_offset) as *const [u8; 32];
                if &*root_ptr == target_root {
                    return true;
                }
            }

            // Continue with regular loop for remaining roots
            for i in 8..Self::MAX_ROOTS {
                let root_offset = 8 + i * 32;
                let root_ptr = self.0.add(root_offset) as *const [u8; 32];
                if &*root_ptr == target_root {
                    return true;
                }
            }
        }
        false
    }
}

/// NullifierShard: Storage for used nullifiers
/// Layout: [count: u32][n * 32-byte nullifiers]
pub struct NullifierShard(*mut u8);

impl NullifierShard {
    pub const MIN_SIZE: usize = 4; // Just the count field
    pub const MAX_NULLIFIERS: usize = 319; // Limited by 10KB CPI realloc cap

    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<Self, ProgramError> {
        if account_info.data_len() < Self::MIN_SIZE {
            return Err(ShieldPoolError::InvalidAccountSize.into());
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn count(&self) -> u32 {
        unsafe { u32::from_le(*(self.0 as *const u32)) }
    }

    #[inline(always)]
    pub fn contains_nullifier(&self, nf: &[u8; 32]) -> bool {
        let count = self.count() as usize;
        unsafe {
            // Unrolled first 4 comparisons for common cases
            let unroll_count = core::cmp::min(4, count);
            for i in 0..unroll_count {
                let nf_offset = 4 + i * 32;
                let stored_nf_ptr = self.0.add(nf_offset) as *const [u8; 32];
                if &*stored_nf_ptr == nf {
                    return true;
                }
            }

            // Continue with regular loop for remaining nullifiers
            for i in unroll_count..count {
                let nf_offset = 4 + i * 32;
                let stored_nf_ptr = self.0.add(nf_offset) as *const [u8; 32];
                if &*stored_nf_ptr == nf {
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn add_nullifier(&mut self, nf: &[u8; 32]) -> Result<(), ProgramError> {
        let count = self.count() as usize;

        // Check capacity
        if count >= Self::MAX_NULLIFIERS {
            return Err(ShieldPoolError::NullifierShardFull.into());
        }

        unsafe {
            // Add nullifier
            let nf_offset = 4 + count * 32;
            let nf_ptr = self.0.add(nf_offset);
            core::ptr::copy_nonoverlapping(nf.as_ptr(), nf_ptr, 32);

            // Update count
            let new_count = (count + 1) as u32;
            *(self.0 as *mut u32) = new_count.to_le();
        }

        Ok(())
    }
}
