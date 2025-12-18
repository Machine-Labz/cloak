use crate::config::Config;
use crate::error::{IndexerError, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawArtifacts {
    pub guest_elf_url: String,
    pub vk: String, // Base64 encoded verification key
    pub sha256: ArtifactHashes,
    pub sp1_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHashes {
    pub elf: String,
    pub vk: String,
}

#[derive(Debug, Clone)]
pub struct ArtifactFileData {
    pub data: Vec<u8>,
    pub sha256: String,
    pub content_type: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactVerification {
    pub valid: bool,
    pub details: ArtifactVerificationDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactVerificationDetails {
    pub elf: bool,
    pub vk: bool,
}

#[derive(Clone)]
pub struct ArtifactManager {
    base_path: PathBuf,
    sp1_version: String,
}

impl ArtifactManager {
    pub fn new(config: &Config) -> Self {
        Self {
            base_path: config.artifacts.base_path.clone(),
            sp1_version: config.artifacts.sp1_version.clone(),
        }
    }

    /// Get artifact information for a specific version
    pub async fn get_withdraw_artifacts(&self, version: &str) -> Result<WithdrawArtifacts> {
        tracing::info!(version = version, "Fetching withdraw artifacts");

        // Construct file paths
        let guest_elf_path = self.get_artifact_path(version, "guest.elf");
        let vk_path = self.get_artifact_path(version, "verification.key");

        // Check if files exist
        if !guest_elf_path.exists() {
            return Err(IndexerError::artifact(format!(
                "Guest ELF not found for version {}: {}",
                version,
                guest_elf_path.display()
            )));
        }

        if !vk_path.exists() {
            return Err(IndexerError::artifact(format!(
                "Verification key not found for version {}: {}",
                version,
                vk_path.display()
            )));
        }

        // Read files and compute hashes
        let guest_elf_data = fs::read(&guest_elf_path).await?;
        let vk_data = fs::read(&vk_path).await?;

        let guest_elf_hash = compute_sha256(&guest_elf_data);
        let vk_hash = compute_sha256(&vk_data);

        // Convert verification key to base64 string for JSON response
        let vk_base64 = base64::prelude::BASE64_STANDARD.encode(&vk_data);

        // Generate URLs
        let guest_elf_url = self.generate_artifact_url(version, "guest.elf");

        let artifacts = WithdrawArtifacts {
            guest_elf_url,
            vk: vk_base64,
            sha256: ArtifactHashes {
                elf: guest_elf_hash.clone(),
                vk: vk_hash.clone(),
            },
            sp1_version: self.sp1_version.clone(),
        };

        tracing::info!(
            version = version,
            guest_elf_url = %artifacts.guest_elf_url,
            vk_length = artifacts.vk.len(),
            elf_hash = %guest_elf_hash[..16],
            vk_hash = %vk_hash[..16],
            "Withdraw artifacts prepared"
        );

        Ok(artifacts)
    }

    /// Serve artifact file content
    pub async fn serve_artifact_file(
        &self,
        version: &str,
        filename: &str,
    ) -> Result<ArtifactFileData> {
        let file_path = self.get_artifact_path(version, filename);

        if !file_path.exists() {
            return Err(IndexerError::artifact(format!(
                "Artifact file not found: {}",
                file_path.display()
            )));
        }

        let data = fs::read(&file_path).await?;
        let sha256 = compute_sha256(&data);
        let content_type = get_content_type(filename);
        let size = data.len();

        tracing::info!(
            version = version,
            filename = filename,
            size = size,
            sha256 = %sha256[..16],
            content_type = %content_type,
            "Serving artifact file"
        );

        Ok(ArtifactFileData {
            data,
            sha256,
            content_type,
            size,
        })
    }

    /// List all available artifact versions
    pub async fn list_available_versions(&self) -> Result<Vec<String>> {
        let mut versions = Vec::new();

        if self.base_path.exists() {
            let mut entries = fs::read_dir(&self.base_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with('v') && name.contains('.') {
                            versions.push(name.to_string());
                        }
                    }
                }
            }
        }

        // If no versions found, return the configured version
        if versions.is_empty() {
            versions.push(self.sp1_version.clone());
        }

        versions.sort();
        Ok(versions)
    }

    /// Verify artifact integrity
    pub async fn verify_artifacts(
        &self,
        version: &str,
        expected_hashes: &ArtifactHashes,
    ) -> Result<ArtifactVerification> {
        match self.get_withdraw_artifacts(version).await {
            Ok(artifacts) => {
                let elf_valid = artifacts.sha256.elf == expected_hashes.elf;
                let vk_valid = artifacts.sha256.vk == expected_hashes.vk;

                tracing::info!(
                    version = version,
                    elf_valid = elf_valid,
                    vk_valid = vk_valid,
                    expected_elf_hash = %expected_hashes.elf[..16],
                    actual_elf_hash = %artifacts.sha256.elf[..16],
                    expected_vk_hash = %expected_hashes.vk[..16],
                    actual_vk_hash = %artifacts.sha256.vk[..16],
                    "Artifact verification completed"
                );

                Ok(ArtifactVerification {
                    valid: elf_valid && vk_valid,
                    details: ArtifactVerificationDetails {
                        elf: elf_valid,
                        vk: vk_valid,
                    },
                })
            }
            Err(e) => {
                tracing::error!(
                    version = version,
                    error = %e,
                    "Artifact verification failed"
                );
                Ok(ArtifactVerification {
                    valid: false,
                    details: ArtifactVerificationDetails {
                        elf: false,
                        vk: false,
                    },
                })
            }
        }
    }

    /// Create placeholder artifacts for development/testing
    pub async fn create_placeholder_artifacts(&self, version: &str) -> Result<()> {
        tracing::info!(
            version = version,
            "Creating placeholder artifacts for development"
        );

        let guest_elf_path = self.get_artifact_path(version, "guest.elf");
        let vk_path = self.get_artifact_path(version, "verification.key");

        // Create version directory if it doesn't exist
        let version_dir = guest_elf_path
            .parent()
            .ok_or_else(|| IndexerError::artifact("Invalid artifact path: no parent directory".to_string()))?;
        fs::create_dir_all(version_dir).await?;

        // Create placeholder guest ELF (dummy binary data)
        if !guest_elf_path.exists() {
            let placeholder_elf = format!(
                "PLACEHOLDER_SP1_GUEST_ELF_{}_{}",
                version,
                chrono::Utc::now().timestamp()
            )
            .repeat(32)
            .into_bytes();

            fs::write(&guest_elf_path, &placeholder_elf).await?;
            tracing::info!(
                path = %guest_elf_path.display(),
                size = placeholder_elf.len(),
                "Created placeholder guest ELF"
            );
        }

        // Create placeholder verification key
        if !vk_path.exists() {
            let placeholder_vk = format!(
                "PLACEHOLDER_VERIFICATION_KEY_{}_{}",
                version,
                chrono::Utc::now().timestamp()
            )
            .repeat(16)
            .into_bytes();

            fs::write(&vk_path, &placeholder_vk).await?;
            tracing::info!(
                path = %vk_path.display(),
                size = placeholder_vk.len(),
                "Created placeholder verification key"
            );
        }

        Ok(())
    }

    /// Get the full path to an artifact file
    fn get_artifact_path(&self, version: &str, filename: &str) -> PathBuf {
        self.base_path.join(version).join(filename)
    }

    /// Generate URL for artifact file
    fn generate_artifact_url(&self, version: &str, filename: &str) -> String {
        format!("/api/v1/artifacts/files/{}/{}", version, filename)
    }
}

/// Compute SHA-256 hash of data
fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Determine content type based on filename
fn get_content_type(filename: &str) -> String {
    if filename.ends_with(".elf") {
        "application/octet-stream".to_string()
    } else if filename.ends_with(".key") || filename.ends_with(".vk") {
        "application/octet-stream".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

// Tests can be added here when needed
