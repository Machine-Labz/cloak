use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::constants::*;

pub struct DepositIx(*mut u8);

impl DepositIx {
    pub const LEN: usize = 8 + 32 + 2;

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        assert_eq!(*account_info.owner(), crate::ID);
        assert_eq!(account_info.data_len(), Self::LEN);
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        unsafe { *(self.0.add(0) as *const u64) }
    }

    #[inline(always)]
    pub fn leaf_commit(&self) -> [u8; 32] {
        unsafe { *(self.0.add(8) as *const [u8; 32]) }
    }

    #[inline(always)]
    pub fn enc_output_len(&self) -> u16 {
        unsafe { *(self.0.add(40) as *const u16) }
    }

    #[inline(always)]
    pub fn enc_output(&self) -> Vec<u8> {
        unsafe {
            Vec::from_raw_parts(
                self.0.add(42) as *mut u8,
                self.enc_output_len() as usize,
                self.enc_output_len() as usize,
            )
        }
    }
}

pub struct AdminPushRootIx(*mut u8);

impl AdminPushRootIx {
    pub const LEN: usize = 32;

    #[inline(always)]
    pub fn from_instruction_data(instruction_data: &[u8]) -> Self {
        unsafe { Self(instruction_data.as_ptr() as *mut u8) }
    }

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    #[inline(always)]
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        assert_eq!(*account_info.owner(), crate::ID);
        assert_eq!(account_info.data_len(), Self::LEN);
        Ok(Self::from_account_info_unchecked(account_info))
    }

    #[inline(always)]
    pub fn new_root(&self) -> [u8; 32] {
        unsafe { *(self.0.add(0) as *const [u8; 32]) }
    }
}

pub struct WithdrawIx(*mut u8);

pub struct WithdrawOutput(*mut u8);

impl WithdrawOutput {
    pub const LEN: usize = PUBKEY_SIZE + 8;

    #[inline(always)]
    pub fn from(ptr: *mut u8) -> Self {
        unsafe { Self(ptr) }
    }

    #[inline(always)]
    pub fn recipient(&self) -> Pubkey {
        unsafe { *(self.0.add(0) as *const Pubkey) }
    }

    #[inline(always)]
    pub fn amount(&self) -> u64 {
        unsafe { *(self.0.add(PUBKEY_SIZE) as *const u64) }
    }
}

impl WithdrawIx {
    pub const LEN: usize =
        SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 3 + 8 + 2 + 1 + PUBKEY_SIZE * 2;

    #[inline(always)]
    pub fn from_instruction_data(instruction_data: &[u8]) -> Self {
        unsafe { Self(instruction_data.as_ptr() as *mut u8) }
    }

    #[inline(always)]
    pub fn sp1_proof(&self) -> [u8; SP1_PROOF_SIZE] {
        unsafe { *(self.0.add(0) as *const [u8; SP1_PROOF_SIZE]) }
    }

    #[inline(always)]
    pub fn sp1_public_inputs(&self) -> [u8; SP1_PUBLIC_INPUTS_SIZE] {
        unsafe { *(self.0.add(SP1_PROOF_SIZE) as *const [u8; SP1_PUBLIC_INPUTS_SIZE]) }
    }

    #[inline(always)]
    pub fn public_root(&self) -> [u8; HASH_SIZE] {
        unsafe { *(self.0.add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE) as *const [u8; HASH_SIZE]) }
    }

    #[inline(always)]
    pub fn public_nf(&self) -> [u8; HASH_SIZE] {
        unsafe {
            *(self
                .0
                .add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE)
                as *const [u8; HASH_SIZE])
        }
    }

    #[inline(always)]
    pub fn public_amount(&self) -> u64 {
        unsafe {
            *(self
                .0
                .add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 2)
                as *const u64)
        }
    }

    #[inline(always)]
    pub fn public_fee_bps(&self) -> u16 {
        unsafe {
            *(self
                .0
                .add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 2 + 8)
                as *const u16)
        }
    }

    #[inline(always)]
    pub fn public_outputs_hash(&self) -> [u8; HASH_SIZE] {
        unsafe {
            *(self
                .0
                .add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 2 + 8 + 2)
                as *const [u8; HASH_SIZE])
        }
    }

    #[inline(always)]
    pub fn outputs(&self) -> Vec<WithdrawOutput> {
        let outputs_len = unsafe {
            *(self
                .0
                .add(SP1_PROOF_SIZE + SP1_PUBLIC_INPUTS_SIZE + HASH_SIZE * 2 + 8 + 2 + 1)
                as *const u8)
        };
        let outputs_len = outputs_len as usize;
        let mut outputs = Vec::with_capacity(outputs_len);
        for i in 0..outputs_len {
            outputs.push(WithdrawOutput::from(unsafe {
                self.0.add(
                    SP1_PROOF_SIZE
                        + SP1_PUBLIC_INPUTS_SIZE
                        + HASH_SIZE * 2
                        + 8
                        + 2
                        + 1
                        + i * WithdrawOutput::LEN,
                )
            }));
        }
        outputs
    }
}
