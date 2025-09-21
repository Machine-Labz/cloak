use pinocchio::program_error::ProgramError;

use crate::error::ShieldPoolError;

/// NullifierShard: Storage for used nullifiers
/// Layout: [count: u32][n * 32-byte nullifiers]
pub struct NullifierShard(*mut u8);

impl NullifierShard {
    pub const MIN_SIZE: usize = 4; // Just the count field
    pub const MAX_NULLIFIERS: usize = 1000; // Reasonable limit for MVP

    #[inline(always)]
    pub fn from_account_data(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < Self::MIN_SIZE {
            return Err(ShieldPoolError::BadAccounts.into());
        }
        Ok(Self(data.as_ptr() as *mut u8))
    }

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
    pub fn init(&mut self) {
        unsafe {
            // Initialize with count = 0
            *(self.0 as *mut u32) = 0u32.to_le();
        }
    }

    #[inline(always)]
    pub fn count(&self) -> u32 {
        unsafe { u32::from_le(*(self.0 as *const u32)) }
    }

    #[inline(always)]
    pub fn contains_nullifier(&self, nf: &[u8; 32]) -> bool {
        let count = self.count() as usize;
        unsafe {
            for i in 0..count {
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
