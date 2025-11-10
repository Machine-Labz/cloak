/**
 * Storage Interface
 * 
 * Defines a pluggable storage interface for notes and keys.
 * Applications can implement their own storage (localStorage, IndexedDB, file system, etc.)
 */

import { CloakNote } from "./types";
import { CloakKeyPair, exportKeys, importKeys } from "./keys";

/**
 * Storage adapter interface
 * 
 * Implement this interface to provide custom storage for notes and keys.
 * The SDK will use this adapter for all persistence operations.
 */
export interface StorageAdapter {
  /**
   * Save a note
   */
  saveNote(note: CloakNote): Promise<void> | void;
  
  /**
   * Load all notes
   */
  loadAllNotes(): Promise<CloakNote[]> | CloakNote[];
  
  /**
   * Update a note
   */
  updateNote(commitment: string, updates: Partial<CloakNote>): Promise<void> | void;
  
  /**
   * Delete a note
   */
  deleteNote(commitment: string): Promise<void> | void;
  
  /**
   * Clear all notes
   */
  clearAllNotes(): Promise<void> | void;
  
  /**
   * Save wallet keys
   */
  saveKeys(keys: CloakKeyPair): Promise<void> | void;
  
  /**
   * Load wallet keys
   */
  loadKeys(): Promise<CloakKeyPair | null> | CloakKeyPair | null;
  
  /**
   * Delete wallet keys
   */
  deleteKeys(): Promise<void> | void;
}

/**
 * In-memory storage adapter (default, no persistence)
 * 
 * Useful for testing or when storage is handled externally
 */
export class MemoryStorageAdapter implements StorageAdapter {
  private notes: Map<string, CloakNote> = new Map();
  private keys: CloakKeyPair | null = null;

  saveNote(note: CloakNote): void {
    this.notes.set(note.commitment, note);
  }

  loadAllNotes(): CloakNote[] {
    return Array.from(this.notes.values());
  }

  updateNote(commitment: string, updates: Partial<CloakNote>): void {
    const existing = this.notes.get(commitment);
    if (existing) {
      this.notes.set(commitment, { ...existing, ...updates });
    }
  }

  deleteNote(commitment: string): void {
    this.notes.delete(commitment);
  }

  clearAllNotes(): void {
    this.notes.clear();
  }

  saveKeys(keys: CloakKeyPair): void {
    this.keys = keys;
  }

  loadKeys(): CloakKeyPair | null {
    return this.keys;
  }

  deleteKeys(): void {
    this.keys = null;
  }
}

/**
 * Browser localStorage adapter (optional, for browser environments)
 * 
 * Only use this if you're in a browser environment and want localStorage persistence.
 * Import from a separate browser-specific module.
 */
export class LocalStorageAdapter implements StorageAdapter {
  private notesKey: string;
  private keysKey: string;

  constructor(notesKey: string = "cloak_notes", keysKey: string = "cloak_wallet_keys") {
    this.notesKey = notesKey;
    this.keysKey = keysKey;
  }

  private getStorage(): Storage | null {
    if (typeof globalThis !== "undefined" && globalThis.localStorage) {
      return globalThis.localStorage;
    }
    return null;
  }

  saveNote(note: CloakNote): void {
    const storage = this.getStorage();
    if (!storage) throw new Error("localStorage not available");
    
    const notes = this.loadAllNotes();
    notes.push(note);
    storage.setItem(this.notesKey, JSON.stringify(notes));
  }

  loadAllNotes(): CloakNote[] {
    const storage = this.getStorage();
    if (!storage) return [];
    
    const stored = storage.getItem(this.notesKey);
    if (!stored) return [];
    
    try {
      return JSON.parse(stored);
    } catch {
      return [];
    }
  }

  updateNote(commitment: string, updates: Partial<CloakNote>): void {
    const storage = this.getStorage();
    if (!storage) return;
    
    const notes = this.loadAllNotes();
    const index = notes.findIndex((n) => n.commitment === commitment);
    
    if (index !== -1) {
      notes[index] = { ...notes[index], ...updates };
      storage.setItem(this.notesKey, JSON.stringify(notes));
    }
  }

  deleteNote(commitment: string): void {
    const storage = this.getStorage();
    if (!storage) return;
    
    const notes = this.loadAllNotes();
    const filtered = notes.filter((n) => n.commitment !== commitment);
    storage.setItem(this.notesKey, JSON.stringify(filtered));
  }

  clearAllNotes(): void {
    const storage = this.getStorage();
    if (storage) {
      storage.removeItem(this.notesKey);
    }
  }

  saveKeys(keys: CloakKeyPair): void {
    const storage = this.getStorage();
    if (!storage) throw new Error("localStorage not available");
    
    storage.setItem(this.keysKey, exportKeys(keys));
  }

  loadKeys(): CloakKeyPair | null {
    const storage = this.getStorage();
    if (!storage) return null;
    
    const stored = storage.getItem(this.keysKey);
    if (!stored) return null;
    
    try {
      return importKeys(stored);
    } catch {
      return null;
    }
  }

  deleteKeys(): void {
    const storage = this.getStorage();
    if (storage) {
      storage.removeItem(this.keysKey);
    }
  }
}
