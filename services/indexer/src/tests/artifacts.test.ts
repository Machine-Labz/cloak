/// <reference types="jest" />
import { ArtifactManager } from '../lib/artifacts.js';
import { join } from 'path';
import { mkdirSync, rmSync, writeFileSync, existsSync } from 'fs';
import { createHash } from 'crypto';

describe('ArtifactManager', () => {
  let artifactManager: ArtifactManager;
  let testArtifactsPath: string;

  beforeEach(() => {
    // Create a temporary test directory
    testArtifactsPath = join(process.cwd(), 'test_artifacts');
    artifactManager = new ArtifactManager(testArtifactsPath);
    
    // Clean up if exists and recreate
    if (existsSync(testArtifactsPath)) {
      rmSync(testArtifactsPath, { recursive: true });
    }
    mkdirSync(testArtifactsPath, { recursive: true });
  });

  afterEach(() => {
    // Clean up test directory
    if (existsSync(testArtifactsPath)) {
      rmSync(testArtifactsPath, { recursive: true });
    }
  });

  describe('Placeholder Creation', () => {
    it('should create placeholder artifacts for development', async () => {
      const version = 'v2.0.0';
      
      await artifactManager.createPlaceholderArtifacts(version);
      
      const guestElfPath = join(testArtifactsPath, version, 'guest.elf');
      const vkPath = join(testArtifactsPath, version, 'verification.key');
      
      expect(existsSync(guestElfPath)).toBe(true);
      expect(existsSync(vkPath)).toBe(true);
    });

    it('should not overwrite existing artifacts', async () => {
      const version = 'v2.0.0';
      const versionDir = join(testArtifactsPath, version);
      const guestElfPath = join(versionDir, 'guest.elf');
      
      // Create version directory and existing file
      mkdirSync(versionDir, { recursive: true });
      const originalContent = Buffer.from('existing_content');
      writeFileSync(guestElfPath, originalContent);
      
      await artifactManager.createPlaceholderArtifacts(version);
      
      // File should not be overwritten
      const content = require('fs').readFileSync(guestElfPath);
      expect(content.toString()).toBe('existing_content');
    });
  });

  describe('Artifact Retrieval', () => {
    beforeEach(async () => {
      // Set up test artifacts
      const version = 'v2.0.0';
      await artifactManager.createPlaceholderArtifacts(version);
    });

    it('should retrieve withdraw artifacts with correct structure', async () => {
      const version = 'v2.0.0';
      
      const artifacts = await artifactManager.getWithdrawArtifacts(version);
      
      expect(artifacts).toHaveProperty('guestElfUrl');
      expect(artifacts).toHaveProperty('vk');
      expect(artifacts).toHaveProperty('sha256');
      expect(artifacts).toHaveProperty('sp1Version');
      
      expect(artifacts.sha256).toHaveProperty('elf');
      expect(artifacts.sha256).toHaveProperty('vk');
      
      expect(artifacts.guestElfUrl).toContain(version);
      expect(artifacts.vk).toBeDefined();
      expect(artifacts.sha256.elf).toMatch(/^[a-f0-9]{64}$/);
      expect(artifacts.sha256.vk).toMatch(/^[a-f0-9]{64}$/);
    });

    it('should throw error for non-existent version', async () => {
      const nonExistentVersion = 'v999.0.0';
      
      await expect(
        artifactManager.getWithdrawArtifacts(nonExistentVersion)
      ).rejects.toThrow('Guest ELF not found');
    });

    it('should compute correct SHA-256 hashes', async () => {
      const version = 'v2.0.0';
      const versionDir = join(testArtifactsPath, version);
      const guestElfPath = join(versionDir, 'guest.elf');
      
      // Create test file with known content
      mkdirSync(versionDir, { recursive: true });
      const testContent = Buffer.from('test_content_for_hash');
      writeFileSync(guestElfPath, testContent);
      writeFileSync(join(versionDir, 'verification.key'), Buffer.from('test_vk'));
      
      const expectedHash = createHash('sha256').update(testContent).digest('hex');
      
      const artifacts = await artifactManager.getWithdrawArtifacts(version);
      
      expect(artifacts.sha256.elf).toBe(expectedHash);
    });
  });

  describe('File Serving', () => {
    beforeEach(async () => {
      const version = 'v2.0.0';
      await artifactManager.createPlaceholderArtifacts(version);
    });

    it('should serve artifact files with correct metadata', async () => {
      const version = 'v2.0.0';
      const filename = 'guest.elf';
      
      const result = await artifactManager.serveArtifactFile(version, filename);
      
      expect(result).toHaveProperty('data');
      expect(result).toHaveProperty('sha256');
      expect(result).toHaveProperty('contentType');
      expect(result).toHaveProperty('size');
      
      expect(Buffer.isBuffer(result.data)).toBe(true);
      expect(result.sha256).toMatch(/^[a-f0-9]{64}$/);
      expect(result.contentType).toBe('application/octet-stream');
      expect(result.size).toBeGreaterThan(0);
    });

    it('should throw error for non-existent files', async () => {
      const version = 'v2.0.0';
      const filename = 'non_existent_file.elf';
      
      await expect(
        artifactManager.serveArtifactFile(version, filename)
      ).rejects.toThrow('Artifact file not found');
    });
  });

  describe('Artifact Verification', () => {
    beforeEach(async () => {
      const version = 'v2.0.0';
      await artifactManager.createPlaceholderArtifacts(version);
    });

    it('should verify artifacts with correct hashes', async () => {
      const version = 'v2.0.0';
      
      // Get actual hashes
      const artifacts = await artifactManager.getWithdrawArtifacts(version);
      const expectedHashes = {
        elf: artifacts.sha256.elf,
        vk: artifacts.sha256.vk,
      };
      
      const verification = await artifactManager.verifyArtifacts(version, expectedHashes);
      
      expect(verification.valid).toBe(true);
      expect(verification.details.elf).toBe(true);
      expect(verification.details.vk).toBe(true);
    });

    it('should fail verification with incorrect hashes', async () => {
      const version = 'v2.0.0';
      const wrongHashes = {
        elf: 'a'.repeat(64),
        vk: 'b'.repeat(64),
      };
      
      const verification = await artifactManager.verifyArtifacts(version, wrongHashes);
      
      expect(verification.valid).toBe(false);
      expect(verification.details.elf).toBe(false);
      expect(verification.details.vk).toBe(false);
    });

    it('should handle verification of non-existent artifacts', async () => {
      const nonExistentVersion = 'v999.0.0';
      const dummyHashes = {
        elf: 'a'.repeat(64),
        vk: 'b'.repeat(64),
      };
      
      const verification = await artifactManager.verifyArtifacts(nonExistentVersion, dummyHashes);
      
      expect(verification.valid).toBe(false);
      expect(verification.details.elf).toBe(false);
      expect(verification.details.vk).toBe(false);
    });
  });

  describe('Version Management', () => {
    it('should list available versions', async () => {
      const versions = await artifactManager.listAvailableVersions();
      
      expect(Array.isArray(versions)).toBe(true);
      expect(versions.length).toBeGreaterThan(0);
      expect(versions[0]).toMatch(/^v\d+\.\d+\.\d+$/);
    });
  });
});
