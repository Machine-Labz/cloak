//! Network constants for Cloak Miner
//!
//! Defines program IDs for different Solana networks.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Devnet,
    Testnet,
    Localnet,
}

impl Network {
    /// Parse network from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "mainnet" | "mainnet-beta" => Ok(Network::Mainnet),
            "devnet" => Ok(Network::Devnet),
            "testnet" => Ok(Network::Testnet),
            "localnet" | "localhost" => Ok(Network::Localnet),
            _ => Err(format!("Unknown network: {}", s)),
        }
    }

    /// Get default RPC URL for this network
    pub fn default_rpc_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://api.mainnet-beta.solana.com",
            Network::Devnet => "https://api.devnet.solana.com",
            Network::Testnet => "https://api.testnet.solana.com",
            Network::Localnet => "http://localhost:8899",
        }
    }

    /// Get scramble registry program ID for this network
    pub fn scramble_program_id(&self) -> Result<Pubkey, String> {
        match self {
            Network::Mainnet => {
                // TODO: Replace with actual mainnet program ID after deployment
                Pubkey::from_str(MAINNET_SCRAMBLE_PROGRAM_ID)
                    .map_err(|e| format!("Invalid mainnet program ID: {}", e))
            }
            Network::Devnet => {
                // TODO: Replace with actual devnet program ID after deployment
                Pubkey::from_str(DEVNET_SCRAMBLE_PROGRAM_ID)
                    .map_err(|e| format!("Invalid devnet program ID: {}", e))
            }
            Network::Testnet => Pubkey::from_str(TESTNET_SCRAMBLE_PROGRAM_ID)
                .map_err(|e| format!("Invalid testnet program ID: {}", e)),
            Network::Localnet => {
                // Use hardcoded localnet program ID (can override with env var)
                let program_id_str = std::env::var("SCRAMBLE_PROGRAM_ID")
                    .unwrap_or_else(|_| LOCALNET_SCRAMBLE_PROGRAM_ID.to_string());

                Pubkey::from_str(&program_id_str)
                    .map_err(|e| format!("Invalid localnet program ID: {}", e))
            }
        }
    }
}

// Program IDs for different networks

/// Mainnet scramble registry program ID
/// TODO: Replace with actual program ID after mainnet deployment
const MAINNET_SCRAMBLE_PROGRAM_ID: &str = "11111111111111111111111111111111";

/// Devnet scramble registry program ID
/// TODO: Replace with actual program ID after devnet deployment
const DEVNET_SCRAMBLE_PROGRAM_ID: &str = "11111111111111111111111111111111";

/// Testnet scramble registry program ID (deployed 2025-10-19)
const TESTNET_SCRAMBLE_PROGRAM_ID: &str = "EH2FoBqySD7RhPgsmPBK67jZ2P9JRhVHjfdnjxhUQEE6";

/// Localnet scramble registry program ID (from build-sbf)
const LOCALNET_SCRAMBLE_PROGRAM_ID: &str = "scramb1eReg1stryPoWM1n1ngSo1anaC1oak11111111";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_from_str() {
        assert_eq!(Network::from_str("mainnet").unwrap(), Network::Mainnet);
        assert_eq!(Network::from_str("mainnet-beta").unwrap(), Network::Mainnet);
        assert_eq!(Network::from_str("devnet").unwrap(), Network::Devnet);
        assert_eq!(Network::from_str("testnet").unwrap(), Network::Testnet);
        assert_eq!(Network::from_str("localnet").unwrap(), Network::Localnet);
        assert_eq!(Network::from_str("localhost").unwrap(), Network::Localnet);

        assert!(Network::from_str("invalid").is_err());
    }

    #[test]
    fn test_default_rpc_urls() {
        assert_eq!(
            Network::Mainnet.default_rpc_url(),
            "https://api.mainnet-beta.solana.com"
        );
        assert_eq!(
            Network::Devnet.default_rpc_url(),
            "https://api.devnet.solana.com"
        );
        assert_eq!(
            Network::Testnet.default_rpc_url(),
            "https://api.testnet.solana.com"
        );
        assert_eq!(Network::Localnet.default_rpc_url(), "http://localhost:8899");
    }

    #[test]
    fn test_program_ids_parse() {
        // Should parse without error
        assert!(Network::Mainnet.scramble_program_id().is_ok());
        assert!(Network::Devnet.scramble_program_id().is_ok());
        assert!(Network::Testnet.scramble_program_id().is_ok());
        assert!(Network::Localnet.scramble_program_id().is_ok());
    }
}
