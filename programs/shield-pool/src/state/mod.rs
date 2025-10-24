use crate::error::ShieldPoolError;
use pinocchio::program_error::ProgramError;

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
    pub const MAX_NULLIFIERS: usize = 1000; // Reasonable limit for MVP

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
