use std::sync::Arc;
use std::time::Duration;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::Transaction,
};
use tracing::{debug, error, info};

use crate::AppState;

// Ore program constants (from ore/api/src/consts.rs)
const INTERMISSION_SLOTS: u64 = 35; // ~14 seconds (must match on-chain value!)
const ONE_MINUTE_SLOTS: u64 = 150; // ~60 seconds

// Ore program state sizes (in bytes, from the program)
const BOARD_SIZE: usize = 8 + 24; // discriminator + Board struct
const ROUND_SIZE: usize = 8 + std::mem::size_of::<RoundData>();

/// Board account structure (from ore/api/src/state/board.rs)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BoardData {
    pub round_id: u64,
    pub start_slot: u64,
    pub end_slot: u64,
}

/// Round account structure (from ore/api/src/state/round.rs)
#[repr(C)]
#[derive(Debug, Clone)]
struct RoundData {
    pub id: u64,
    pub deployed: [u64; 25],
    pub slot_hash: [u8; 32],
    pub count: [u64; 25],
    pub expires_at: u64,
    pub motherlode: u64,
    pub rent_payer: Pubkey,
    pub top_miner: Pubkey,
    pub top_miner_reward: u64,
    pub total_deployed: u64,
    pub total_vaulted: u64,
    pub total_winnings: u64,
}

/// Configuration for the Ore Round Manager
#[derive(Clone, Debug)]
pub struct OreRoundManagerConfig {
    /// Ore program ID
    pub program_id: Pubkey,
    /// How often to check round status (in seconds)
    pub poll_interval_secs: u64,
    /// Optional authority keypair for triggering resets (Arc for sharing)
    pub authority_keypair: Option<Arc<Keypair>>,
    /// Enable auto-reset functionality
    pub auto_reset_enabled: bool,
}

impl Default for OreRoundManagerConfig {
    fn default() -> Self {
        Self {
            // Default to devnet deployment
            program_id: "3xkMEM9BsKo3gS9PBkKHHfjcQ1VDHV8eSyGfsi5LmqHB"
                .parse()
                .unwrap(),
            poll_interval_secs: 10, // Check every 10 seconds
            authority_keypair: None,
            auto_reset_enabled: false, // Disabled by default for safety
        }
    }
}

/// Monitors Ore rounds and manages round lifecycle
pub struct OreRoundManager {
    state: AppState,
    config: OreRoundManagerConfig,
    last_seen_round: Arc<tokio::sync::Mutex<u64>>,
}

impl OreRoundManager {
    pub fn new(state: AppState, config: OreRoundManagerConfig) -> Self {
        Self {
            state,
            config,
            last_seen_round: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Start the round manager loop
    pub async fn run(self: Arc<Self>) {
        info!("🎯 Ore Round Manager started");
        info!("   Program ID: {}", self.config.program_id);
        info!("   Poll interval: {}s", self.config.poll_interval_secs);
        info!(
            "   Auto-reset: {}",
            if self.config.auto_reset_enabled {
                "ENABLED"
            } else {
                "DISABLED (monitoring only)"
            }
        );

        let poll_interval = Duration::from_secs(self.config.poll_interval_secs);

        loop {
            if let Err(e) = self.check_round_status().await {
                error!("❌ Error checking round status: {}", e);
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Check the current round status and take action if needed
    async fn check_round_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get current slot
        let current_slot = self.state.solana.get_slot().await?;

        // Get board account
        let board_pda = self.get_board_pda();
        let board = match self.fetch_board(&board_pda).await {
            Ok(b) => b,
            Err(e) => {
                debug!("Board account not found or invalid: {}", e);
                return Ok(());
            }
        };

        // Check if we've seen a new round
        let mut last_round = self.last_seen_round.lock().await;
        if board.round_id != *last_round {
            info!(
                "🔄 Round changed: {} → {} (slot: {})",
                *last_round, board.round_id, current_slot
            );
            *last_round = board.round_id;
        }
        drop(last_round);

        // Calculate round state
        let round_state = self.calculate_round_state(&board, current_slot);

        match round_state {
            RoundState::WaitingForFirstDeploy => {
                debug!(
                    "Round {} - waiting for first deploy to start (end_slot: {})",
                    board.round_id, board.end_slot
                );
            }
            RoundState::Active {
                slots_remaining,
                seconds_remaining,
            } => {
                debug!(
                    "Round {} - ACTIVE | {} slots remaining (~{}s) | ends at slot {}",
                    board.round_id, slots_remaining, seconds_remaining, board.end_slot
                );
            }
            RoundState::Intermission { slots_until_reset } => {
                info!(
                    "⏳ Round {} - INTERMISSION | {} slots until reset | current slot: {} | end slot: {}",
                    board.round_id, slots_until_reset, current_slot, board.end_slot
                );
            }
            RoundState::ReadyForReset => {
                info!(
                    "✅ Round {} - READY FOR RESET | current slot: {} | end slot: {}",
                    board.round_id, current_slot, board.end_slot
                );

                // Attempt to reset if auto-reset is enabled
                if self.config.auto_reset_enabled {
                    if let Err(e) = self.trigger_reset(&board).await {
                        error!("Failed to trigger reset for round {}: {}", board.round_id, e);
                    }
                } else {
                    debug!("Auto-reset disabled, skipping automatic reset");
                }
            }
        }

        Ok(())
    }

    /// Fetch and parse the Board account
    async fn fetch_board(&self, board_pda: &Pubkey) -> Result<BoardData, Box<dyn std::error::Error>> {
        let account = self.state.solana.get_account(board_pda).await?;

        if account.data.len() < BOARD_SIZE {
            return Err(format!(
                "Board account too small: {} bytes (expected {})",
                account.data.len(),
                BOARD_SIZE
            )
            .into());
        }

        // Skip 8-byte discriminator, parse Board struct
        let data = &account.data[8..];
        let round_id = u64::from_le_bytes(data[0..8].try_into()?);
        let start_slot = u64::from_le_bytes(data[8..16].try_into()?);
        let end_slot = u64::from_le_bytes(data[16..24].try_into()?);

        Ok(BoardData {
            round_id,
            start_slot,
            end_slot,
        })
    }

    /// Calculate the current round state
    fn calculate_round_state(&self, board: &BoardData, current_slot: u64) -> RoundState {
        // If end_slot is u64::MAX, round hasn't started yet
        if board.end_slot == u64::MAX {
            return RoundState::WaitingForFirstDeploy;
        }

        // If current slot is before end_slot, round is active
        if current_slot < board.end_slot {
            let slots_remaining = board.end_slot - current_slot;
            let seconds_remaining = (slots_remaining * 60) / ONE_MINUTE_SLOTS;
            return RoundState::Active {
                slots_remaining,
                seconds_remaining,
            };
        }

        // Round has ended, check if intermission is over
        let intermission_end = board.end_slot + INTERMISSION_SLOTS;
        if current_slot < intermission_end {
            let slots_until_reset = intermission_end - current_slot;
            return RoundState::Intermission { slots_until_reset };
        }

        // Ready for reset
        RoundState::ReadyForReset
    }

    /// Trigger a round reset transaction
    async fn trigger_reset(&self, board: &BoardData) -> Result<(), Box<dyn std::error::Error>> {
        let authority = match &self.config.authority_keypair {
            Some(kp) => kp,
            None => {
                return Err("Authority keypair not configured for auto-reset".into());
            }
        };

        info!("📤 Triggering reset for round {}...", board.round_id);

        // Build reset instruction (simplified - you'll need to add all required accounts)
        let reset_ix = self.build_reset_instruction(&authority.pubkey(), board)?;

        // Build and send transaction
        let blockhash = self
            .state
            .solana
            .get_latest_blockhash()
            .await
            .map_err(|e| format!("Failed to get blockhash: {}", e))?;

        let transaction = Transaction::new_signed_with_payer(
            &[reset_ix],
            Some(&authority.pubkey()),
            &[authority],
            blockhash,
        );

        match self.state.solana.send_and_confirm_transaction(&transaction).await {
            Ok(sig) => {
                info!("✅ Reset successful! Signature: {}", sig);
                Ok(())
            }
            Err(e) => {
                error!("❌ Reset transaction failed: {}", e);
                Err(e.into())
            }
        }
    }

    /// Build the reset instruction
    fn build_reset_instruction(
        &self,
        authority: &Pubkey,
        board: &BoardData,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        // Get all required PDAs
        let board_pda = self.get_board_pda();
        let config_pda = self.get_config_pda();
        let round_pda = self.get_round_pda(board.round_id);
        let round_next_pda = self.get_round_pda(board.round_id + 1);
        let treasury_pda = self.get_treasury_pda();

        // ORE mint address (from ore/api/src/consts.rs)
        let mint_address: Pubkey = "DCmGtyAJNeDyvQ7JM2hNBW3uLE1jhcW8BQzoNvWVvKbE".parse()?;

        // Get treasury's associated token account
        let treasury_tokens = spl_associated_token_account::get_associated_token_address(
            &treasury_pda,
            &mint_address,
        );

        // Get fee collector from config (using default for now - you may want to fetch this)
        let fee_collector = *authority; // Simplified - ideally fetch from config account

        // Top miner (use default since we're not validating yet)
        let top_miner_pda = Pubkey::default();

        // Build the reset instruction
        // Instruction discriminator for Reset (you'll need to match ore/program)
        // Steel framework: 1-byte discriminator (Reset = 9 from OreInstruction enum)
        // Reset struct is empty, so just the discriminator byte
        let instruction_data = vec![9];

        let instruction = Instruction {
            program_id: self.config.program_id,
            accounts: vec![
                AccountMeta::new(*authority, true),              // signer
                AccountMeta::new(board_pda, false),              // board
                AccountMeta::new(config_pda, false),             // config
                AccountMeta::new(fee_collector, false),          // fee_collector
                AccountMeta::new(mint_address, false),           // mint
                AccountMeta::new(round_pda, false),              // round
                AccountMeta::new(round_next_pda, false),         // round_next
                AccountMeta::new(top_miner_pda, false),          // top_miner
                AccountMeta::new(treasury_pda, false),           // treasury
                AccountMeta::new(treasury_tokens, false),        // treasury_tokens
                AccountMeta::new_readonly(system_program::ID, false), // system_program
                AccountMeta::new_readonly(spl_token::ID, false), // token_program
                AccountMeta::new_readonly(self.config.program_id, false), // ore_program
                AccountMeta::new_readonly(sysvar::slot_hashes::ID, false), // slot_hashes_sysvar
                // Entropy accounts (pass defaults for devnet)
                AccountMeta::new(Pubkey::default(), false),      // var (placeholder)
                AccountMeta::new_readonly(Pubkey::default(), false), // entropy_program (placeholder)
            ],
            data: instruction_data,
        };

        Ok(instruction)
    }

    /// Get the Treasury PDA
    fn get_treasury_pda(&self) -> Pubkey {
        let (pda, _bump) = Pubkey::find_program_address(&[b"treasury"], &self.config.program_id);
        pda
    }

    /// Get the Board PDA
    fn get_board_pda(&self) -> Pubkey {
        let (pda, _bump) = Pubkey::find_program_address(&[b"board"], &self.config.program_id);
        pda
    }

    /// Get the Round PDA for a specific round ID
    fn get_round_pda(&self, round_id: u64) -> Pubkey {
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"round", &round_id.to_le_bytes()],
            &self.config.program_id,
        );
        pda
    }

    /// Get the Config PDA
    fn get_config_pda(&self) -> Pubkey {
        let (pda, _bump) = Pubkey::find_program_address(&[b"config"], &self.config.program_id);
        pda
    }
}

/// Round state enum
#[derive(Debug, Clone)]
enum RoundState {
    /// Waiting for first deploy (end_slot = u64::MAX)
    WaitingForFirstDeploy,
    /// Round is active and mining
    Active {
        slots_remaining: u64,
        seconds_remaining: u64,
    },
    /// Round ended, in intermission period
    Intermission { slots_until_reset: u64 },
    /// Ready for reset transaction
    ReadyForReset,
}
