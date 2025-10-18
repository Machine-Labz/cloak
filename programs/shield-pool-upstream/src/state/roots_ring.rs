use pinocchio::program_error::ProgramError;

use crate::error::ShieldPoolError;

/// RootsRing: Fixed-size ring buffer of recent Merkle roots
/// Layout: [head: u8][pad: 7][roots: 64 * 32 bytes] => total = 8 + 2048 = 2056 bytes
pub struct RootsRing(*mut u8);

impl RootsRing {
    pub const SIZE: usize = 8 + 64 * 32; // 2056 bytes
    pub const MAX_ROOTS: usize = 64;

    #[inline(always)]
    pub fn from_account_data(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() != Self::SIZE {
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
        if account_info.data_len() != Self::SIZE {
            return Err(ShieldPoolError::InvalidAccountSize.into());
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn init(&mut self) {
        unsafe {
            // Initialize with head = 0, all roots = 0
            core::ptr::write_bytes(self.0, 0, Self::SIZE);
        }
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
            for i in 0..Self::MAX_ROOTS {
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
