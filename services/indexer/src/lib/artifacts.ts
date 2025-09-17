import { createHash } from 'crypto';
import { readFileSync, existsSync, statSync } from 'fs';
import { join, dirname } from 'path';
import { WithdrawArtifacts } from '../types/index.js';
import { logger } from './logger.js';
import { config } from './config.js';

/**
 * Artifact management for SP1 guest ELF and verification keys
 * Handles file serving with SHA-256 integrity checks
 */
export class ArtifactManager {
  private readonly basePath: string;

  constructor(basePath?: string) {
    this.basePath = basePath || config.artifacts.basePath;
    logger.info('ArtifactManager initialized', { basePath: this.basePath });
  }

  /**
   * Get artifact information for a specific version
   * @param version - Version string (e.g., "v2.0.0")
   * @returns Promise<WithdrawArtifacts>
   */
  async getWithdrawArtifacts(version: string): Promise<WithdrawArtifacts> {
    logger.info('Fetching withdraw artifacts', { version });

    try {
      // Construct file paths
      const guestElfPath = this.getArtifactPath(version, 'guest.elf');
      const vkPath = this.getArtifactPath(version, 'verification.key');

      // Check if files exist
      if (!existsSync(guestElfPath)) {
        throw new Error(`Guest ELF not found for version ${version}: ${guestElfPath}`);
      }

      if (!existsSync(vkPath)) {
        throw new Error(`Verification key not found for version ${version}: ${vkPath}`);
      }

      // Read files and compute hashes
      const guestElfData = readFileSync(guestElfPath);
      const vkData = readFileSync(vkPath);

      const guestElfHash = this.computeSHA256(guestElfData);
      const vkHash = this.computeSHA256(vkData);

      // Convert verification key to base64 string for JSON response
      const vkBase64 = vkData.toString('base64');

      // Generate URLs (in production, these would be proper HTTP URLs)
      const guestElfUrl = this.generateArtifactUrl(version, 'guest.elf');

      const artifacts: WithdrawArtifacts = {
        guestElfUrl,
        vk: vkBase64,
        sha256: {
          elf: guestElfHash,
          vk: vkHash,
        },
        sp1Version: config.artifacts.sp1Version,
      };

      logger.info('Withdraw artifacts prepared', {
        version,
        guestElfUrl,
        vkLength: vkBase64.length,
        elfHash: guestElfHash.substring(0, 16) + '...',
        vkHashPrefix: vkHash.substring(0, 16) + '...',
      });

      return artifacts;
    } catch (error) {
      logger.error('Failed to get withdraw artifacts', { version, error });
      throw error;
    }
  }

  /**
   * Serve artifact file content
   * @param version - Version string
   * @param filename - Filename (e.g., "guest.elf", "verification.key")
   * @returns Buffer containing file data
   */
  async serveArtifactFile(version: string, filename: string): Promise<{ 
    data: Buffer; 
    sha256: string; 
    contentType: string;
    size: number;
  }> {
    const filePath = this.getArtifactPath(version, filename);

    if (!existsSync(filePath)) {
      throw new Error(`Artifact file not found: ${filePath}`);
    }

    try {
      const data = readFileSync(filePath);
      const sha256 = this.computeSHA256(data);
      const stats = statSync(filePath);
      const contentType = this.getContentType(filename);

      logger.info('Serving artifact file', {
        version,
        filename,
        size: data.length,
        sha256: sha256.substring(0, 16) + '...',
        contentType,
      });

      return {
        data,
        sha256,
        contentType,
        size: stats.size,
      };
    } catch (error) {
      logger.error('Failed to serve artifact file', { version, filename, filePath, error });
      throw error;
    }
  }

  /**
   * List all available artifact versions
   */
  async listAvailableVersions(): Promise<string[]> {
    // This would typically scan the artifacts directory
    // For now, return the configured version
    return [config.artifacts.sp1Version];
  }

  /**
   * Verify artifact integrity
   * @param version - Version to verify
   * @param expectedHashes - Expected SHA-256 hashes
   */
  async verifyArtifacts(
    version: string, 
    expectedHashes: { elf: string; vk: string }
  ): Promise<{ valid: boolean; details: Record<string, boolean> }> {
    try {
      const artifacts = await this.getWithdrawArtifacts(version);
      
      const elfValid = artifacts.sha256.elf === expectedHashes.elf;
      const vkValid = artifacts.sha256.vk === expectedHashes.vk;

      logger.info('Artifact verification completed', {
        version,
        elfValid,
        vkValid,
        expectedElfHash: expectedHashes.elf.substring(0, 16) + '...',
        actualElfHash: artifacts.sha256.elf.substring(0, 16) + '...',
        expectedVkHash: expectedHashes.vk.substring(0, 16) + '...',
        actualVkHash: artifacts.sha256.vk.substring(0, 16) + '...',
      });

      return {
        valid: elfValid && vkValid,
        details: {
          elf: elfValid,
          vk: vkValid,
        },
      };
    } catch (error) {
      logger.error('Artifact verification failed', { version, error });
      return {
        valid: false,
        details: {
          elf: false,
          vk: false,
        },
      };
    }
  }

  /**
   * Get the full path to an artifact file
   */
  private getArtifactPath(version: string, filename: string): string {
    return join(this.basePath, version, filename);
  }

  /**
   * Generate URL for artifact file
   */
  private generateArtifactUrl(version: string, filename: string): string {
    // In production, this would be a full HTTP URL to the artifact server
    return `/api/v1/artifacts/files/${version}/${filename}`;
  }

  /**
   * Compute SHA-256 hash of data
   */
  private computeSHA256(data: Buffer): string {
    return createHash('sha256').update(data).digest('hex');
  }

  /**
   * Determine content type based on filename
   */
  private getContentType(filename: string): string {
    if (filename.endsWith('.elf')) {
      return 'application/octet-stream';
    }
    if (filename.endsWith('.key') || filename.endsWith('.vk')) {
      return 'application/octet-stream';
    }
    return 'application/octet-stream';
  }

  /**
   * Create placeholder artifacts for development/testing
   * This creates dummy files if they don't exist
   */
  async createPlaceholderArtifacts(version: string): Promise<void> {
    logger.info('Creating placeholder artifacts for development', { version });

    const guestElfPath = this.getArtifactPath(version, 'guest.elf');
    const vkPath = this.getArtifactPath(version, 'verification.key');

    // Create version directory if it doesn't exist
    const versionDir = dirname(guestElfPath);
    await this.ensureDirectoryExists(versionDir);

    // Create placeholder guest ELF (dummy binary data)
    if (!existsSync(guestElfPath)) {
      const placeholderElf = Buffer.from(
        `PLACEHOLDER_SP1_GUEST_ELF_${version}_${Date.now()}`.repeat(32)
      );
      await this.writeFile(guestElfPath, placeholderElf);
      logger.info('Created placeholder guest ELF', { path: guestElfPath, size: placeholderElf.length });
    }

    // Create placeholder verification key
    if (!existsSync(vkPath)) {
      const placeholderVk = Buffer.from(
        `PLACEHOLDER_VERIFICATION_KEY_${version}_${Date.now()}`.repeat(16)
      );
      await this.writeFile(vkPath, placeholderVk);
      logger.info('Created placeholder verification key', { path: vkPath, size: placeholderVk.length });
    }
  }

  /**
   * Helper methods for file operations
   */
  private async ensureDirectoryExists(dirPath: string): Promise<void> {
    const { mkdir } = await import('fs/promises');
    try {
      await mkdir(dirPath, { recursive: true });
    } catch (error: any) {
      if (error.code !== 'EEXIST') {
        throw error;
      }
    }
  }

  private async writeFile(filePath: string, data: Buffer): Promise<void> {
    const { writeFile } = await import('fs/promises');
    await writeFile(filePath, data);
  }
}

// Export singleton instance
export const artifactManager = new ArtifactManager();
