"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  CloakError: () => CloakError,
  CloakSDK: () => CloakSDK,
  DepositRecoveryService: () => DepositRecoveryService,
  FIXED_FEE_LAMPORTS: () => FIXED_FEE_LAMPORTS,
  IndexerService: () => IndexerService,
  LAMPORTS_PER_SOL: () => LAMPORTS_PER_SOL,
  LocalStorageAdapter: () => LocalStorageAdapter,
  MemoryStorageAdapter: () => MemoryStorageAdapter,
  ProverService: () => ProverService,
  RelayService: () => RelayService,
  VARIABLE_FEE_RATE: () => VARIABLE_FEE_RATE,
  VERSION: () => VERSION,
  bytesToHex: () => bytesToHex,
  calculateFee: () => calculateFee2,
  calculateRelayFee: () => calculateRelayFee,
  computeNullifier: () => computeNullifier,
  computeOutputsHash: () => computeOutputsHash,
  copyNoteToClipboard: () => copyNoteToClipboard,
  createCloakError: () => createCloakError,
  createDepositInstruction: () => createDepositInstruction,
  deriveSpendKey: () => deriveSpendKey,
  deriveViewKey: () => deriveViewKey,
  detectNetworkFromRpcUrl: () => detectNetworkFromRpcUrl,
  downloadNote: () => downloadNote,
  encodeNoteSimple: () => encodeNoteSimple,
  encryptNoteForRecipient: () => encryptNoteForRecipient,
  exportKeys: () => exportKeys,
  exportNote: () => exportNote,
  exportWalletKeys: () => exportWalletKeys,
  filterNotesByNetwork: () => filterNotesByNetwork,
  filterWithdrawableNotes: () => filterWithdrawableNotes,
  findNoteByCommitment: () => findNoteByCommitment,
  formatAmount: () => formatAmount,
  formatErrorForLogging: () => formatErrorForLogging,
  generateCloakKeys: () => generateCloakKeys,
  generateCommitment: () => generateCommitment,
  generateMasterSeed: () => generateMasterSeed,
  generateNote: () => generateNote,
  generateNoteFromWallet: () => generateNoteFromWallet,
  getAddressExplorerUrl: () => getAddressExplorerUrl,
  getDistributableAmount: () => getDistributableAmount2,
  getExplorerUrl: () => getExplorerUrl,
  getPublicKey: () => getPublicKey,
  getPublicViewKey: () => getPublicViewKey,
  getRecipientAmount: () => getRecipientAmount,
  getRpcUrlForNetwork: () => getRpcUrlForNetwork,
  getViewKey: () => getViewKey,
  hexToBytes: () => hexToBytes,
  importKeys: () => importKeys,
  importWalletKeys: () => importWalletKeys,
  isValidHex: () => isValidHex,
  isValidRpcUrl: () => isValidRpcUrl,
  isValidSolanaAddress: () => isValidSolanaAddress,
  isWithdrawable: () => isWithdrawable,
  keypairToAdapter: () => keypairToAdapter,
  parseAmount: () => parseAmount,
  parseNote: () => parseNote,
  parseTransactionError: () => parseTransactionError,
  prepareEncryptedOutput: () => prepareEncryptedOutput,
  prepareEncryptedOutputForRecipient: () => prepareEncryptedOutputForRecipient,
  randomBytes: () => randomBytes,
  scanNotesForWallet: () => scanNotesForWallet,
  sendTransaction: () => sendTransaction,
  serializeNote: () => serializeNote,
  signTransaction: () => signTransaction,
  tryDecryptNote: () => tryDecryptNote,
  updateNoteWithDeposit: () => updateNoteWithDeposit,
  validateDepositParams: () => validateDepositParams,
  validateNote: () => validateNote,
  validateOutputsSum: () => validateOutputsSum,
  validateTransfers: () => validateTransfers,
  validateWalletConnected: () => validateWalletConnected,
  validateWithdrawableNote: () => validateWithdrawableNote
});
module.exports = __toCommonJS(index_exports);

// src/core/CloakSDK.ts
var import_web35 = require("@solana/web3.js");

// src/core/types.ts
var CloakError = class extends Error {
  constructor(message, category, retryable = false, originalError) {
    super(message);
    this.category = category;
    this.retryable = retryable;
    this.originalError = originalError;
    this.name = "CloakError";
  }
};

// src/core/note-manager.ts
var import_blake33 = require("@noble/hashes/blake3.js");
var import_buffer = require("buffer");

// src/core/keys.ts
var import_blake32 = require("@noble/hashes/blake3.js");
var import_tweetnacl = __toESM(require("tweetnacl"), 1);

// src/utils/crypto.ts
var import_blake3 = require("@noble/hashes/blake3.js");
function generateCommitment(amountLamports, r, skSpend) {
  const pkSpend = (0, import_blake3.blake3)(skSpend);
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);
  const commitmentInput = new Uint8Array(72);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(r, 8);
  commitmentInput.set(pkSpend, 40);
  return (0, import_blake3.blake3)(commitmentInput);
}
function computeNullifier(skSpend, leafIndex) {
  const leafIndexBytes = new Uint8Array(4);
  new DataView(leafIndexBytes.buffer).setUint32(0, leafIndex, true);
  const nullifierInput = new Uint8Array(36);
  nullifierInput.set(skSpend, 0);
  nullifierInput.set(leafIndexBytes, 32);
  return (0, import_blake3.blake3)(nullifierInput);
}
function computeOutputsHash(outputs) {
  const chunks = [];
  for (const output of outputs) {
    chunks.push(output.recipient.toBytes());
    const amountBytes = new Uint8Array(8);
    new DataView(amountBytes.buffer).setBigUint64(0, BigInt(output.amount), true);
    chunks.push(amountBytes);
  }
  const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    combined.set(chunk, offset);
    offset += chunk.length;
  }
  return (0, import_blake3.blake3)(combined);
}
function hexToBytes(hex) {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;
  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
  }
  return bytes;
}
function bytesToHex(bytes, prefix = false) {
  const hex = Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
  return prefix ? `0x${hex}` : hex;
}
function randomBytes(length) {
  const bytes = new Uint8Array(length);
  const g = globalThis;
  try {
    const cryptoObj = g?.crypto || g?.window?.crypto || g?.self?.crypto;
    if (cryptoObj && typeof cryptoObj.getRandomValues === "function") {
      cryptoObj.getRandomValues(bytes);
      return bytes;
    }
  } catch {
  }
  try {
    const nodeCrypto = require("crypto");
    if (nodeCrypto?.randomBytes) {
      const buffer = nodeCrypto.randomBytes(length);
      bytes.set(buffer);
      return bytes;
    }
    if (nodeCrypto?.webcrypto?.getRandomValues) {
      nodeCrypto.webcrypto.getRandomValues(bytes);
      return bytes;
    }
  } catch {
  }
  for (let i = 0; i < length; i++) bytes[i] = Math.floor(Math.random() * 256);
  return bytes;
}
function isValidHex(hex, expectedLength) {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (!/^[0-9a-f]*$/i.test(cleanHex)) {
    return false;
  }
  if (cleanHex.length % 2 !== 0) {
    return false;
  }
  if (expectedLength !== void 0) {
    return cleanHex.length === expectedLength * 2;
  }
  return true;
}

// src/core/keys.ts
function generateMasterSeed() {
  const seed = new Uint8Array(32);
  const g = globalThis;
  const cryptoObj = g?.crypto || g?.window?.crypto || g?.self?.crypto;
  if (cryptoObj && typeof cryptoObj.getRandomValues === "function") {
    cryptoObj.getRandomValues(seed);
  } else {
    try {
      const nodeCrypto = require("crypto");
      const buffer = nodeCrypto.randomBytes(32);
      seed.set(buffer);
    } catch {
      throw new Error("No secure random number generator available");
    }
  }
  return {
    seed,
    seedHex: bytesToHex(seed)
  };
}
function deriveSpendKey(masterSeed) {
  const context = new TextEncoder().encode("cloak_spend_key");
  const preimage = new Uint8Array(masterSeed.length + context.length);
  preimage.set(masterSeed, 0);
  preimage.set(context, masterSeed.length);
  const sk_spend = (0, import_blake32.blake3)(preimage);
  const pk_spend = (0, import_blake32.blake3)(sk_spend);
  return {
    sk_spend,
    pk_spend,
    sk_spend_hex: bytesToHex(sk_spend),
    pk_spend_hex: bytesToHex(pk_spend)
  };
}
function deriveViewKey(sk_spend) {
  const context = new TextEncoder().encode("cloak_view_key_secret");
  const preimage = new Uint8Array(sk_spend.length + context.length);
  preimage.set(sk_spend, 0);
  preimage.set(context, sk_spend.length);
  const vk_secret = (0, import_blake32.blake3)(preimage);
  const x25519Keypair = import_tweetnacl.default.box.keyPair.fromSecretKey(vk_secret);
  return {
    vk_secret,
    pvk: x25519Keypair.publicKey,
    vk_secret_hex: bytesToHex(vk_secret),
    pvk_hex: bytesToHex(x25519Keypair.publicKey)
  };
}
function generateCloakKeys(masterSeed) {
  const master = masterSeed ? { seed: masterSeed, seedHex: bytesToHex(masterSeed) } : generateMasterSeed();
  const spend = deriveSpendKey(master.seed);
  const view = deriveViewKey(spend.sk_spend);
  return {
    master,
    spend,
    view
  };
}
function encryptNoteForRecipient(noteData, recipientPvk) {
  const ephemeralKeypair = import_tweetnacl.default.box.keyPair();
  const sharedSecret = import_tweetnacl.default.box.before(recipientPvk, ephemeralKeypair.secretKey);
  const plaintext = new TextEncoder().encode(JSON.stringify(noteData));
  const nonce = import_tweetnacl.default.randomBytes(import_tweetnacl.default.secretbox.nonceLength);
  const ciphertext = import_tweetnacl.default.secretbox(plaintext, nonce, sharedSecret);
  return {
    ephemeral_pk: bytesToHex(ephemeralKeypair.publicKey),
    ciphertext: bytesToHex(ciphertext),
    nonce: bytesToHex(nonce)
  };
}
function tryDecryptNote(encryptedNote, viewKey) {
  try {
    const ephemeralPk = hexToBytes(encryptedNote.ephemeral_pk);
    const ciphertext = hexToBytes(encryptedNote.ciphertext);
    const nonce = hexToBytes(encryptedNote.nonce);
    const x25519Secret = viewKey.vk_secret;
    const sharedSecret = import_tweetnacl.default.box.before(ephemeralPk, x25519Secret);
    const plaintext = import_tweetnacl.default.secretbox.open(ciphertext, nonce, sharedSecret);
    if (!plaintext) {
      return null;
    }
    const noteData = JSON.parse(new TextDecoder().decode(plaintext));
    return noteData;
  } catch (e) {
    return null;
  }
}
function scanNotesForWallet(encryptedOutputs, viewKey) {
  const foundNotes = [];
  for (const encryptedOutput of encryptedOutputs) {
    try {
      const decoded = atob(encryptedOutput);
      const encryptedNote = JSON.parse(decoded);
      const noteData = tryDecryptNote(encryptedNote, viewKey);
      if (noteData) {
        foundNotes.push(noteData);
      }
    } catch (e) {
      continue;
    }
  }
  return foundNotes;
}
function exportKeys(keys) {
  return JSON.stringify({
    version: "2.0",
    master_seed: keys.master.seedHex,
    sk_spend: keys.spend.sk_spend_hex,
    pk_spend: keys.spend.pk_spend_hex,
    vk_secret: keys.view.vk_secret_hex,
    pvk: keys.view.pvk_hex
  }, null, 2);
}
function importKeys(exported) {
  const parsed = JSON.parse(exported);
  const masterSeed = hexToBytes(parsed.master_seed);
  return generateCloakKeys(masterSeed);
}

// src/utils/network.ts
function detectNetworkFromRpcUrl(rpcUrl) {
  const url = rpcUrl || process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "";
  const lowerUrl = url.toLowerCase();
  if (lowerUrl.includes("mainnet") || lowerUrl.includes("api.mainnet-beta") || lowerUrl.includes("mainnet-beta")) {
    return "mainnet";
  }
  if (lowerUrl.includes("testnet") || lowerUrl.includes("api.testnet")) {
    return "testnet";
  }
  if (lowerUrl.includes("devnet") || lowerUrl.includes("api.devnet")) {
    return "devnet";
  }
  if (lowerUrl.includes("localhost") || lowerUrl.includes("127.0.0.1") || lowerUrl.includes("local")) {
    return "localnet";
  }
  return "devnet";
}
function getRpcUrlForNetwork(network) {
  switch (network) {
    case "mainnet":
      return "https://api.mainnet-beta.solana.com";
    case "testnet":
      return "https://api.testnet.solana.com";
    case "devnet":
      return "https://api.devnet.solana.com";
    case "localnet":
      return "http://localhost:8899";
    default:
      return "https://api.devnet.solana.com";
  }
}
function isValidRpcUrl(url) {
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}
function getExplorerUrl(signature, network = "devnet") {
  const cluster = network === "mainnet" ? "" : `?cluster=${network}`;
  return `https://explorer.solana.com/tx/${signature}${cluster}`;
}
function getAddressExplorerUrl(address, network = "devnet") {
  const cluster = network === "mainnet" ? "" : `?cluster=${network}`;
  return `https://explorer.solana.com/address/${address}${cluster}`;
}

// src/core/note-manager.ts
function generateNote(amountLamports, network) {
  const actualNetwork = network || detectNetworkFromRpcUrl();
  const skSpend = new Uint8Array(32);
  const rBytes = new Uint8Array(32);
  crypto.getRandomValues(skSpend);
  crypto.getRandomValues(rBytes);
  const pkSpend = (0, import_blake33.blake3)(skSpend);
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);
  const commitmentInput = new Uint8Array(8 + 32 + 32);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(rBytes, 8);
  commitmentInput.set(pkSpend, 40);
  const commitmentBytes = (0, import_blake33.blake3)(commitmentInput);
  const skSpendHex = import_buffer.Buffer.from(skSpend).toString("hex");
  const rHex = import_buffer.Buffer.from(rBytes).toString("hex");
  const commitmentHex = import_buffer.Buffer.from(commitmentBytes).toString("hex");
  return {
    version: "1.0",
    amount: amountLamports,
    commitment: commitmentHex,
    sk_spend: skSpendHex,
    r: rHex,
    timestamp: Date.now(),
    network: actualNetwork
  };
}
function generateNoteFromWallet(amountLamports, keys, network) {
  const actualNetwork = network || detectNetworkFromRpcUrl();
  const rBytes = new Uint8Array(32);
  crypto.getRandomValues(rBytes);
  const pk_spend = import_buffer.Buffer.from(keys.spend.pk_spend_hex, "hex");
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, BigInt(amountLamports), true);
  const commitmentInput = new Uint8Array(8 + 32 + 32);
  commitmentInput.set(amountBytes, 0);
  commitmentInput.set(rBytes, 8);
  commitmentInput.set(pk_spend, 40);
  const commitmentBytes = (0, import_blake33.blake3)(commitmentInput);
  return {
    version: "2.0",
    amount: amountLamports,
    commitment: import_buffer.Buffer.from(commitmentBytes).toString("hex"),
    sk_spend: keys.spend.sk_spend_hex,
    r: import_buffer.Buffer.from(rBytes).toString("hex"),
    timestamp: Date.now(),
    network: actualNetwork
  };
}
function parseNote(jsonString) {
  const note = JSON.parse(jsonString);
  if (!note.version || !note.amount || !note.commitment || !note.sk_spend || !note.r) {
    throw new CloakError(
      "Invalid note format: missing required fields",
      "validation",
      false
    );
  }
  if (!/^[0-9a-f]{64}$/i.test(note.commitment)) {
    throw new CloakError("Invalid commitment format", "validation", false);
  }
  if (!/^[0-9a-f]{64}$/i.test(note.sk_spend)) {
    throw new CloakError("Invalid sk_spend format", "validation", false);
  }
  if (!/^[0-9a-f]{64}$/i.test(note.r)) {
    throw new CloakError("Invalid r format", "validation", false);
  }
  return note;
}
function exportNote(note, pretty = false) {
  return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
}
function isWithdrawable(note) {
  return !!(note.depositSignature && note.leafIndex !== void 0 && note.root && note.merkleProof);
}
function updateNoteWithDeposit(note, depositInfo) {
  return {
    ...note,
    depositSignature: depositInfo.signature,
    depositSlot: depositInfo.slot,
    leafIndex: depositInfo.leafIndex,
    root: depositInfo.root,
    merkleProof: depositInfo.merkleProof
  };
}
function findNoteByCommitment(notes, commitment) {
  return notes.find((n) => n.commitment === commitment);
}
function filterNotesByNetwork(notes, network) {
  return notes.filter((n) => n.network === network);
}
function filterWithdrawableNotes(notes) {
  return notes.filter(isWithdrawable);
}
function exportWalletKeys(keys) {
  return exportKeys(keys);
}
function importWalletKeys(keysJson) {
  return importKeys(keysJson);
}
function getPublicViewKey(keys) {
  return keys.view.pvk_hex;
}
function getViewKey(keys) {
  return keys.view;
}
function calculateFee(amountLamports) {
  const FIXED_FEE_LAMPORTS2 = 25e5;
  const variableFee = Math.floor(amountLamports * 5 / 1e3);
  return FIXED_FEE_LAMPORTS2 + variableFee;
}
function getDistributableAmount(amountLamports) {
  return amountLamports - calculateFee(amountLamports);
}
function getRecipientAmount(amountLamports) {
  return getDistributableAmount(amountLamports);
}

// src/core/storage.ts
var MemoryStorageAdapter = class {
  constructor() {
    this.notes = /* @__PURE__ */ new Map();
    this.keys = null;
  }
  saveNote(note) {
    this.notes.set(note.commitment, note);
  }
  loadAllNotes() {
    return Array.from(this.notes.values());
  }
  updateNote(commitment, updates) {
    const existing = this.notes.get(commitment);
    if (existing) {
      this.notes.set(commitment, { ...existing, ...updates });
    }
  }
  deleteNote(commitment) {
    this.notes.delete(commitment);
  }
  clearAllNotes() {
    this.notes.clear();
  }
  saveKeys(keys) {
    this.keys = keys;
  }
  loadKeys() {
    return this.keys;
  }
  deleteKeys() {
    this.keys = null;
  }
};
var LocalStorageAdapter = class {
  constructor(notesKey = "cloak_notes", keysKey = "cloak_wallet_keys") {
    this.notesKey = notesKey;
    this.keysKey = keysKey;
  }
  getStorage() {
    if (typeof globalThis !== "undefined" && globalThis.localStorage) {
      return globalThis.localStorage;
    }
    return null;
  }
  saveNote(note) {
    const storage = this.getStorage();
    if (!storage) throw new Error("localStorage not available");
    const notes = this.loadAllNotes();
    notes.push(note);
    storage.setItem(this.notesKey, JSON.stringify(notes));
  }
  loadAllNotes() {
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
  updateNote(commitment, updates) {
    const storage = this.getStorage();
    if (!storage) return;
    const notes = this.loadAllNotes();
    const index = notes.findIndex((n) => n.commitment === commitment);
    if (index !== -1) {
      notes[index] = { ...notes[index], ...updates };
      storage.setItem(this.notesKey, JSON.stringify(notes));
    }
  }
  deleteNote(commitment) {
    const storage = this.getStorage();
    if (!storage) return;
    const notes = this.loadAllNotes();
    const filtered = notes.filter((n) => n.commitment !== commitment);
    storage.setItem(this.notesKey, JSON.stringify(filtered));
  }
  clearAllNotes() {
    const storage = this.getStorage();
    if (storage) {
      storage.removeItem(this.notesKey);
    }
  }
  saveKeys(keys) {
    const storage = this.getStorage();
    if (!storage) throw new Error("localStorage not available");
    storage.setItem(this.keysKey, exportKeys(keys));
  }
  loadKeys() {
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
  deleteKeys() {
    const storage = this.getStorage();
    if (storage) {
      storage.removeItem(this.keysKey);
    }
  }
};

// src/utils/validation.ts
var import_web3 = require("@solana/web3.js");
function isValidSolanaAddress(address) {
  try {
    new import_web3.PublicKey(address);
    return true;
  } catch {
    return false;
  }
}
function validateNote(note) {
  if (!note || typeof note !== "object") {
    throw new Error("Note must be an object");
  }
  const requiredFields = ["version", "amount", "commitment", "sk_spend", "r", "timestamp", "network"];
  for (const field of requiredFields) {
    if (!(field in note)) {
      throw new Error(`Missing required field: ${field}`);
    }
  }
  if (typeof note.version !== "string") {
    throw new Error("Version must be a string");
  }
  if (typeof note.amount !== "number" || note.amount <= 0) {
    throw new Error("Amount must be a positive number");
  }
  if (!isValidHex(note.commitment, 32)) {
    throw new Error("Invalid commitment format (expected 64 hex characters)");
  }
  if (!isValidHex(note.sk_spend, 32)) {
    throw new Error("Invalid sk_spend format (expected 64 hex characters)");
  }
  if (!isValidHex(note.r, 32)) {
    throw new Error("Invalid r format (expected 64 hex characters)");
  }
  if (typeof note.timestamp !== "number" || note.timestamp <= 0) {
    throw new Error("Timestamp must be a positive number");
  }
  if (!["localnet", "devnet", "testnet", "mainnet"].includes(note.network)) {
    throw new Error("Network must be localnet, devnet, testnet, or mainnet");
  }
  if (note.depositSignature !== void 0 && typeof note.depositSignature !== "string") {
    throw new Error("Deposit signature must be a string");
  }
  if (note.depositSlot !== void 0 && typeof note.depositSlot !== "number") {
    throw new Error("Deposit slot must be a number");
  }
  if (note.leafIndex !== void 0) {
    if (typeof note.leafIndex !== "number" || note.leafIndex < 0) {
      throw new Error("Leaf index must be a non-negative number");
    }
  }
  if (note.root !== void 0 && !isValidHex(note.root, 32)) {
    throw new Error("Invalid root format (expected 64 hex characters)");
  }
  if (note.merkleProof !== void 0) {
    if (!Array.isArray(note.merkleProof.pathElements)) {
      throw new Error("Merkle proof pathElements must be an array");
    }
    if (!Array.isArray(note.merkleProof.pathIndices)) {
      throw new Error("Merkle proof pathIndices must be an array");
    }
    if (note.merkleProof.pathElements.length !== note.merkleProof.pathIndices.length) {
      throw new Error("Merkle proof pathElements and pathIndices must have same length");
    }
  }
}
function parseNote2(jsonString) {
  let parsed;
  try {
    parsed = JSON.parse(jsonString);
  } catch (error) {
    throw new Error("Invalid JSON format");
  }
  validateNote(parsed);
  return parsed;
}
function validateWithdrawableNote(note) {
  if (!note.depositSignature) {
    throw new Error("Note must be deposited before withdrawal (missing depositSignature)");
  }
  if (note.leafIndex === void 0) {
    throw new Error("Note must be deposited before withdrawal (missing leafIndex)");
  }
  if (!note.root) {
    throw new Error("Note must have historical root for withdrawal");
  }
  if (!note.merkleProof) {
    throw new Error("Note must have Merkle proof for withdrawal");
  }
  if (note.merkleProof.pathElements.length === 0) {
    throw new Error("Merkle proof is empty");
  }
}
function validateTransfers(recipients, totalAmount) {
  if (recipients.length === 0) {
    throw new Error("At least one recipient is required");
  }
  if (recipients.length > 5) {
    throw new Error("Maximum 5 recipients allowed");
  }
  for (let i = 0; i < recipients.length; i++) {
    const transfer = recipients[i];
    if (!transfer.recipient || !(transfer.recipient instanceof import_web3.PublicKey)) {
      throw new Error(`Recipient ${i} must be a PublicKey`);
    }
    if (typeof transfer.amount !== "number" || transfer.amount <= 0) {
      throw new Error(`Recipient ${i} amount must be a positive number`);
    }
  }
  const sum = recipients.reduce((acc, t) => acc + t.amount, 0);
  if (sum !== totalAmount) {
    throw new Error(
      `Recipients sum (${sum}) does not match expected total (${totalAmount})`
    );
  }
}

// src/utils/fees.ts
var FIXED_FEE_LAMPORTS = 25e5;
var VARIABLE_FEE_RATE = 5 / 1e3;
var LAMPORTS_PER_SOL = 1e9;
function calculateFee2(amountLamports) {
  const variableFee = Math.floor(amountLamports * 5 / 1e3);
  return FIXED_FEE_LAMPORTS + variableFee;
}
function getDistributableAmount2(amountLamports) {
  return amountLamports - calculateFee2(amountLamports);
}
function formatAmount(lamports, decimals = 9) {
  return (lamports / LAMPORTS_PER_SOL).toFixed(decimals);
}
function parseAmount(sol) {
  const num = parseFloat(sol);
  if (isNaN(num) || num < 0) {
    throw new Error(`Invalid SOL amount: ${sol}`);
  }
  return Math.floor(num * LAMPORTS_PER_SOL);
}
function validateOutputsSum(outputs, expectedTotal) {
  const sum = outputs.reduce((acc, out) => acc + out.amount, 0);
  return sum === expectedTotal;
}
function calculateRelayFee(amountLamports, feeBps) {
  if (feeBps < 0 || feeBps > 1e4) {
    throw new Error("Fee basis points must be between 0 and 10000");
  }
  return Math.floor(amountLamports * feeBps / 1e4);
}

// src/services/IndexerService.ts
var IndexerService = class {
  /**
   * Create a new Indexer Service client
   *
   * @param baseUrl - Indexer API base URL
   */
  constructor(baseUrl) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
  }
  /**
   * Get current Merkle root and next available index
   *
   * @returns Current root and next index
   *
   * @example
   * ```typescript
   * const { root, next_index } = await indexer.getMerkleRoot();
   * console.log(`Current root: ${root}, Next index: ${next_index}`);
   * ```
   */
  async getMerkleRoot() {
    const response = await fetch(`${this.baseUrl}/api/v1/merkle/root`);
    if (!response.ok) {
      throw new Error(
        `Failed to get Merkle root: ${response.status} ${response.statusText}`
      );
    }
    const json = await response.json();
    return json;
  }
  /**
   * Get Merkle proof for a specific leaf
   *
   * @param leafIndex - Index of the leaf in the tree
   * @returns Merkle proof with path elements and indices
   *
   * @example
   * ```typescript
   * const proof = await indexer.getMerkleProof(42);
   * console.log(`Proof has ${proof.pathElements.length} siblings`);
   * ```
   */
  async getMerkleProof(leafIndex) {
    const response = await fetch(
      `${this.baseUrl}/api/v1/merkle/proof/${leafIndex}`
    );
    if (!response.ok) {
      throw new Error(
        `Failed to get Merkle proof: ${response.status} ${response.statusText}`
      );
    }
    const data = await response.json();
    return {
      pathElements: data.pathElements ?? data.path_elements,
      pathIndices: data.pathIndices ?? data.path_indices,
      root: data.root
    };
  }
  /**
   * Get notes in a specific range
   *
   * Useful for scanning the tree or fetching notes in batches.
   *
   * @param start - Start index (inclusive)
   * @param end - End index (inclusive)
   * @param limit - Maximum number of notes to return (default: 100)
   * @returns Notes in the range
   *
   * @example
   * ```typescript
   * const { notes, has_more } = await indexer.getNotesRange(0, 99, 100);
   * console.log(`Fetched ${notes.length} notes`);
   * ```
   */
  async getNotesRange(start, end, limit = 100) {
    const url = new URL(`${this.baseUrl}/api/v1/notes/range`);
    url.searchParams.set("start", start.toString());
    url.searchParams.set("end", end.toString());
    url.searchParams.set("limit", limit.toString());
    const response = await fetch(url.toString());
    if (!response.ok) {
      throw new Error(
        `Failed to get notes range: ${response.status} ${response.statusText}`
      );
    }
    const json = await response.json();
    return json;
  }
  /**
   * Get all notes from the tree
   *
   * Fetches all notes in batches. Use with caution for large trees.
   *
   * @param batchSize - Size of each batch (default: 100)
   * @returns All encrypted notes
   *
   * @example
   * ```typescript
   * const allNotes = await indexer.getAllNotes();
   * console.log(`Total notes: ${allNotes.length}`);
   * ```
   */
  async getAllNotes(batchSize = 100) {
    const rootResponse = await this.getMerkleRoot();
    const totalNotes = rootResponse.next_index;
    if (totalNotes === 0) {
      return [];
    }
    const allNotes = [];
    for (let start = 0; start < totalNotes; start += batchSize) {
      const end = Math.min(start + batchSize - 1, totalNotes - 1);
      const response = await this.getNotesRange(start, end, batchSize);
      allNotes.push(...response.notes);
    }
    return allNotes;
  }
  /**
   * Submit a deposit to the indexer
   *
   * Registers a new deposit transaction with the indexer, which will
   * return the leaf index and current root.
   *
   * @param params - Deposit parameters
   * @returns Success response with leaf index and root
   *
   * @example
   * ```typescript
   * const result = await indexer.submitDeposit({
   *   leafCommit: note.commitment,
   *   encryptedOutput: btoa(JSON.stringify(noteData)),
   *   txSignature: signature,
   *   slot: txSlot
   * });
   * console.log(`Leaf index: ${result.leafIndex}`);
   * ```
   */
  async submitDeposit(params) {
    const response = await fetch(`${this.baseUrl}/api/v1/deposit`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        leaf_commit: params.leafCommit,
        encrypted_output: params.encryptedOutput,
        tx_signature: params.txSignature,
        slot: params.slot
      })
    });
    let responseData;
    try {
      responseData = await response.json();
    } catch {
      try {
        const text = await response.text();
        responseData = text ? { error: text } : null;
      } catch {
        responseData = null;
      }
    }
    if (!response.ok) {
      let errorMessage = `${response.status} ${response.statusText}`;
      if (responseData) {
        if (typeof responseData === "string") {
          errorMessage = responseData;
        } else {
          errorMessage = responseData?.error || responseData?.message || errorMessage;
          if (responseData?.details) {
            errorMessage += ` (${JSON.stringify(responseData.details)})`;
          }
        }
      }
      throw new Error(`Failed to submit deposit: ${errorMessage}`);
    }
    const data = responseData;
    return {
      success: data.success ?? true,
      leafIndex: data.leafIndex ?? data.leaf_index,
      root: data.root
    };
  }
  /**
   * Check indexer health
   *
   * @returns Health status
   */
  async healthCheck() {
    const response = await fetch(`${this.baseUrl}/health`);
    if (!response.ok) {
      throw new Error(
        `Health check failed: ${response.status} ${response.statusText}`
      );
    }
    const json = await response.json();
    return json;
  }
};

// src/services/ProverService.ts
var ProverService = class {
  /**
   * Create a new Prover Service client
   *
   * @param indexerUrl - Indexer/Prover service base URL
   * @param timeout - Proof generation timeout in ms (default: 5 minutes)
   */
  constructor(indexerUrl, timeout = 5 * 60 * 1e3) {
    this.indexerUrl = indexerUrl.replace(/\/$/, "");
    this.timeout = timeout;
  }
  /**
   * Generate a zero-knowledge proof for withdrawal
   *
   * This process typically takes 30-180 seconds depending on the backend.
   *
   * @param inputs - Circuit inputs (private + public + outputs)
   * @param options - Optional progress tracking and callbacks
   * @returns Proof result with hex-encoded proof and public inputs
   *
   * @example
   * ```typescript
   * const result = await prover.generateProof(inputs);
   * if (result.success) {
   *   console.log(`Proof: ${result.proof}`);
   * }
   * ```
   * 
   * @example
   * ```typescript
   * // With progress tracking
   * const result = await prover.generateProof(inputs, {
   *   onProgress: (progress) => console.log(`Progress: ${progress}%`),
   *   onStart: () => console.log("Starting proof generation..."),
   *   onSuccess: (result) => console.log("Proof generated!"),
   *   onError: (error) => console.error("Failed:", error)
   * });
   * ```
   */
  async generateProof(inputs, options) {
    const startTime = Date.now();
    const actualTimeout = options?.timeout || this.timeout;
    options?.onStart?.();
    let progressInterval;
    try {
      const requestBody = {
        private_inputs: JSON.stringify(inputs.privateInputs),
        public_inputs: JSON.stringify(inputs.publicInputs),
        outputs: JSON.stringify(inputs.outputs)
      };
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), actualTimeout);
      if (options?.onProgress) {
        let progress = 0;
        progressInterval = setInterval(() => {
          progress = Math.min(90, progress + Math.random() * 10);
          options.onProgress(Math.floor(progress));
        }, 2e3);
      }
      const response = await fetch(`${this.indexerUrl}/api/v1/prove`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(requestBody),
        signal: controller.signal
      });
      clearTimeout(timeoutId);
      if (progressInterval) clearInterval(progressInterval);
      if (!response.ok) {
        let errorMessage = `${response.status} ${response.statusText}`;
        try {
          const errorText = await response.text();
          try {
            const errorJson = JSON.parse(errorText);
            errorMessage = errorJson.error || errorJson.message || errorText;
          } catch {
            errorMessage = errorText || errorMessage;
          }
        } catch {
        }
        options?.onError?.(errorMessage);
        return {
          success: false,
          generationTimeMs: Date.now() - startTime,
          error: errorMessage
        };
      }
      options?.onProgress?.(100);
      const rawData = await response.json();
      const result = {
        success: rawData.success,
        proof: rawData.proof,
        publicInputs: rawData.public_inputs,
        // Map snake_case
        generationTimeMs: rawData.generation_time_ms || Date.now() - startTime,
        error: rawData.error
      };
      if (!result.success && rawData.execution_report) {
      }
      if (!result.success && result.error) {
        try {
          const errorObj = typeof result.error === "string" ? JSON.parse(result.error) : result.error;
          if (errorObj?.error && typeof errorObj.error === "string") {
            result.error = errorObj.error;
          } else if (typeof errorObj === "string") {
            result.error = errorObj;
          }
          if (errorObj?.execution_report && typeof errorObj.execution_report === "string") {
            result.error += `
Execution report: ${errorObj.execution_report}`;
          }
          if (errorObj?.total_cycles !== void 0) {
            result.error += `
Total cycles: ${errorObj.total_cycles}`;
          }
          if (errorObj?.total_syscalls !== void 0) {
            result.error += `
Total syscalls: ${errorObj.total_syscalls}`;
          }
        } catch {
        }
      }
      if (result.success) {
        options?.onSuccess?.(result);
      } else if (result.error) {
        options?.onError?.(result.error);
      }
      return result;
    } catch (error) {
      const totalTime = Date.now() - startTime;
      if (progressInterval) clearInterval(progressInterval);
      let errorMessage;
      if (error instanceof Error && error.name === "AbortError") {
        errorMessage = `Proof generation timed out after ${actualTimeout}ms`;
      } else {
        errorMessage = error instanceof Error ? error.message : "Unknown error occurred";
      }
      options?.onError?.(errorMessage);
      return {
        success: false,
        generationTimeMs: totalTime,
        error: errorMessage
      };
    }
  }
  /**
   * Check if the prover service is available
   *
   * @returns True if service is healthy
   */
  async healthCheck() {
    try {
      const response = await fetch(`${this.indexerUrl}/health`, {
        method: "GET"
      });
      return response.ok;
    } catch {
      return false;
    }
  }
  /**
   * Get the configured timeout
   */
  getTimeout() {
    return this.timeout;
  }
  /**
   * Set a new timeout
   */
  setTimeout(timeout) {
    if (timeout <= 0) {
      throw new Error("Timeout must be positive");
    }
    this.timeout = timeout;
  }
};

// src/services/RelayService.ts
var RelayService = class {
  /**
   * Create a new Relay Service client
   *
   * @param baseUrl - Relay service base URL
   */
  constructor(baseUrl) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
  }
  /**
   * Submit a withdrawal transaction via relay
   *
   * The relay service will validate the proof, pay for transaction fees,
   * and submit the transaction on-chain.
   *
   * @param params - Withdrawal parameters
   * @param onStatusUpdate - Optional callback for status updates
   * @returns Transaction signature when completed
   *
   * @example
   * ```typescript
   * const signature = await relay.submitWithdraw({
   *   proof: proofHex,
   *   publicInputs: { root, nf, outputs_hash, amount },
   *   outputs: [{ recipient: addr, amount: lamports }],
   *   feeBps: 50
   * }, (status) => console.log(`Status: ${status}`));
   * console.log(`Transaction: ${signature}`);
   * ```
   */
  async submitWithdraw(params, onStatusUpdate) {
    const proofBytes = hexToBytes(params.proof);
    const proofBase64 = this.bytesToBase64(proofBytes);
    const requestBody = {
      outputs: params.outputs,
      policy: {
        fee_bps: params.feeBps
      },
      public_inputs: {
        root: params.publicInputs.root,
        nf: params.publicInputs.nf,
        amount: params.publicInputs.amount,
        fee_bps: params.feeBps,
        outputs_hash: params.publicInputs.outputs_hash
      },
      proof_bytes: proofBase64
    };
    const response = await fetch(`${this.baseUrl}/withdraw`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(requestBody)
    });
    if (!response.ok) {
      let errorMessage = `${response.status} ${response.statusText}`;
      try {
        const errorText = await response.text();
        errorMessage = errorText || errorMessage;
      } catch {
      }
      throw new Error(`Relay withdraw failed: ${errorMessage}`);
    }
    const json = await response.json();
    if (!json.success) {
      throw new Error(json.error || "Relay withdraw failed");
    }
    const requestId = json.data?.request_id;
    if (!requestId) {
      throw new Error("Relay response missing request_id");
    }
    return this.pollForCompletion(requestId, onStatusUpdate);
  }
  /**
   * Poll for withdrawal completion
   *
   * @param requestId - Request ID from relay service
   * @param onStatusUpdate - Optional callback for status updates
   * @returns Transaction signature when completed
   */
  async pollForCompletion(requestId, onStatusUpdate) {
    let attempts = 0;
    const maxAttempts = 120;
    const pollInterval = 5e3;
    while (attempts < maxAttempts) {
      await this.sleep(pollInterval);
      attempts++;
      try {
        const statusResp = await fetch(`${this.baseUrl}/status/${requestId}`);
        if (!statusResp.ok) {
          continue;
        }
        const statusJson = await statusResp.json();
        const statusData = statusJson.data;
        const status = statusData?.status;
        if (onStatusUpdate && status) {
          onStatusUpdate(status);
        }
        if (status === "completed") {
          const txId = statusData?.tx_id;
          if (!txId) {
            throw new Error("Relay completed without tx_id");
          }
          return txId;
        }
        if (status === "failed") {
          throw new Error(statusData?.error || "Relay job failed");
        }
      } catch (error) {
        if (error instanceof Error && error.message.includes("failed")) {
          throw error;
        }
      }
    }
    throw new Error(
      `Withdrawal polling timed out after ${maxAttempts * pollInterval}ms`
    );
  }
  /**
   * Get transaction status
   *
   * @param requestId - Request ID from previous submission
   * @returns Current status
   *
   * @example
   * ```typescript
   * const status = await relay.getStatus(requestId);
   * console.log(`Status: ${status.status}`);
   * if (status.status === 'completed') {
   *   console.log(`TX: ${status.txId}`);
   * }
   * ```
   */
  async getStatus(requestId) {
    const response = await fetch(`${this.baseUrl}/status/${requestId}`);
    if (!response.ok) {
      throw new Error(
        `Failed to get status: ${response.status} ${response.statusText}`
      );
    }
    const json = await response.json();
    const data = json.data;
    return {
      status: data?.status || "pending",
      txId: data?.tx_id,
      error: data?.error
    };
  }
  /**
   * Convert bytes to base64 string
   */
  bytesToBase64(bytes) {
    if (typeof Buffer !== "undefined") {
      return Buffer.from(bytes).toString("base64");
    } else if (typeof btoa !== "undefined") {
      const binary = Array.from(bytes).map((b) => String.fromCharCode(b)).join("");
      return btoa(binary);
    } else {
      throw new Error("No base64 encoding method available");
    }
  }
  /**
   * Sleep utility
   */
  sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
};

// src/services/DepositRecoveryService.ts
var import_web32 = require("@solana/web3.js");

// src/helpers/encrypted-output.ts
function prepareEncryptedOutput(note, cloakKeys) {
  const noteData = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment
  };
  const encrypted = encryptNoteForRecipient(noteData, cloakKeys.view.pvk);
  return btoa(JSON.stringify(encrypted));
}
function prepareEncryptedOutputForRecipient(note, recipientPvkHex) {
  const noteData = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment
  };
  const recipientPvk = hexToBytes(recipientPvkHex);
  const encrypted = encryptNoteForRecipient(noteData, recipientPvk);
  return btoa(JSON.stringify(encrypted));
}
function encodeNoteSimple(note) {
  const data = {
    amount: note.amount,
    r: note.r,
    sk_spend: note.sk_spend,
    commitment: note.commitment
  };
  const json = JSON.stringify(data);
  if (typeof Buffer !== "undefined") {
    return Buffer.from(json).toString("base64");
  } else if (typeof btoa !== "undefined") {
    return btoa(json);
  } else {
    throw new Error("No base64 encoding method available");
  }
}

// src/services/DepositRecoveryService.ts
var DepositRecoveryService = class {
  constructor(indexer, apiUrl) {
    this.indexer = indexer;
    this.apiUrl = apiUrl;
  }
  /**
   * Recover a deposit that completed on-chain but failed to register
   * 
   * @param options Recovery options
   * @returns Recovery result with updated note
   */
  async recoverDeposit(options) {
    const { signature, commitment, note, onProgress } = options;
    try {
      onProgress?.("Validating inputs...");
      if (!/^[1-9A-HJ-NP-Za-km-z]{87,88}$/.test(signature)) {
        throw new CloakError(
          "Invalid transaction signature format",
          "validation",
          false
        );
      }
      if (!/^[0-9a-f]{64}$/i.test(commitment)) {
        throw new CloakError(
          "Invalid commitment format",
          "validation",
          false
        );
      }
      onProgress?.("Checking if deposit is already registered...");
      try {
        const existingInfo = await this.checkExistingDeposit(commitment);
        if (existingInfo) {
          onProgress?.("Deposit already registered!");
          return {
            success: true,
            ...existingInfo,
            note: note ? updateNoteWithDeposit(note, {
              signature,
              slot: existingInfo.slot,
              leafIndex: existingInfo.leafIndex,
              root: existingInfo.root,
              merkleProof: existingInfo.merkleProof
            }) : void 0
          };
        }
      } catch (e) {
      }
      onProgress?.("Fetching transaction details...");
      const connection = new import_web32.Connection(
        process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "https://api.devnet.solana.com"
      );
      const txDetails = await connection.getTransaction(signature, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0
      });
      if (!txDetails) {
        throw new CloakError(
          "Transaction not found on blockchain",
          "validation",
          false
        );
      }
      const slot = txDetails.slot;
      onProgress?.("Preparing encrypted output...");
      let encryptedOutput = "";
      if (note) {
        encryptedOutput = encodeNoteSimple(note);
      } else {
        encryptedOutput = btoa(JSON.stringify({ commitment }));
      }
      onProgress?.("Registering deposit with indexer...");
      const depositResponse = await this.indexer.submitDeposit({
        leafCommit: commitment,
        encryptedOutput,
        txSignature: signature,
        slot
      });
      if (!depositResponse.success) {
        throw new CloakError(
          "Failed to register deposit with indexer",
          "indexer",
          true
        );
      }
      const leafIndex = depositResponse.leafIndex;
      const root = depositResponse.root;
      onProgress?.("Fetching Merkle proof...");
      const merkleProof = await this.indexer.getMerkleProof(leafIndex);
      onProgress?.("Recovery complete!");
      const updatedNote = note ? updateNoteWithDeposit(note, {
        signature,
        slot,
        leafIndex,
        root,
        merkleProof: {
          pathElements: merkleProof.pathElements,
          pathIndices: merkleProof.pathIndices
        }
      }) : void 0;
      return {
        success: true,
        leafIndex,
        root,
        slot,
        merkleProof: {
          pathElements: merkleProof.pathElements,
          pathIndices: merkleProof.pathIndices
        },
        note: updatedNote
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      if (errorMessage.includes("duplicate key") || errorMessage.includes("already exists")) {
        try {
          const existingInfo = await this.checkExistingDeposit(commitment);
          if (existingInfo) {
            return {
              success: true,
              ...existingInfo,
              note: note ? updateNoteWithDeposit(note, {
                signature,
                slot: existingInfo.slot,
                leafIndex: existingInfo.leafIndex,
                root: existingInfo.root,
                merkleProof: existingInfo.merkleProof
              }) : void 0
            };
          }
        } catch (e) {
        }
      }
      return {
        success: false,
        error: errorMessage
      };
    }
  }
  /**
   * Check if a deposit already exists in the indexer
   * 
   * @private
   */
  async checkExistingDeposit(_commitment) {
    try {
      const { next_index } = await this.indexer.getMerkleRoot();
      const batchSize = 100;
      for (let i = 0; i < next_index; i += batchSize) {
        const end = Math.min(i + batchSize - 1, next_index - 1);
        const { notes } = await this.indexer.getNotesRange(i, end, batchSize);
        for (let j = 0; j < notes.length; j++) {
          try {
            return null;
          } catch (e) {
            continue;
          }
        }
      }
      return null;
    } catch (error) {
      return null;
    }
  }
  /**
   * Finalize a deposit via server API (alternative recovery method)
   * 
   * This method calls a server-side endpoint that can handle
   * the recovery process with elevated permissions.
   */
  async finalizeDepositViaServer(signature, commitment, encryptedOutput) {
    try {
      const response = await fetch(`${this.apiUrl}/api/deposit/finalize`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          tx_signature: signature,
          commitment,
          encrypted_output: encryptedOutput || btoa(JSON.stringify({ commitment }))
        })
      });
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Recovery failed: ${errorText}`);
      }
      const data = await response.json();
      if (!data.success) {
        throw new Error(data.error || "Recovery failed");
      }
      if (!data.leaf_index || !data.root || data.slot === void 0 || !data.merkle_proof) {
        throw new Error("Recovery response missing required fields");
      }
      return {
        success: true,
        leafIndex: data.leaf_index,
        root: data.root,
        slot: data.slot,
        merkleProof: {
          pathElements: data.merkle_proof.path_elements,
          pathIndices: data.merkle_proof.path_indices
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }
};

// src/solana/instructions.ts
var import_web33 = require("@solana/web3.js");
function createDepositInstruction(params) {
  if (params.commitment.length !== 32) {
    throw new Error(
      `Invalid commitment length: ${params.commitment.length} (expected 32 bytes)`
    );
  }
  if (params.amount <= 0) {
    throw new Error("Amount must be positive");
  }
  const discriminant = new Uint8Array([0]);
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(
    0,
    BigInt(params.amount),
    true
    // little-endian
  );
  const data = new Uint8Array(41);
  data.set(discriminant, 0);
  data.set(amountBytes, 1);
  data.set(params.commitment, 9);
  return new import_web33.TransactionInstruction({
    programId: params.programId,
    keys: [
      // Account 0: Payer (signer, writable) - pays for transaction
      { pubkey: params.payer, isSigner: true, isWritable: true },
      // Account 1: Pool (writable) - receives SOL
      { pubkey: params.pool, isSigner: false, isWritable: true },
      // Account 2: System Program (readonly) - for transfers
      { pubkey: import_web33.SystemProgram.programId, isSigner: false, isWritable: false },
      // Account 3: Commitments (writable) - stores commitment
      { pubkey: params.commitments, isSigner: false, isWritable: true }
    ],
    data: Buffer.from(data)
  });
}
function validateDepositParams(params) {
  if (!(params.programId instanceof import_web33.PublicKey)) {
    throw new Error("programId must be a PublicKey");
  }
  if (!(params.payer instanceof import_web33.PublicKey)) {
    throw new Error("payer must be a PublicKey");
  }
  if (!(params.pool instanceof import_web33.PublicKey)) {
    throw new Error("pool must be a PublicKey");
  }
  if (!(params.commitments instanceof import_web33.PublicKey)) {
    throw new Error("commitments must be a PublicKey");
  }
  if (typeof params.amount !== "number" || params.amount <= 0) {
    throw new Error("amount must be a positive number");
  }
  if (!(params.commitment instanceof Uint8Array)) {
    throw new Error("commitment must be a Uint8Array");
  }
  if (params.commitment.length !== 32) {
    throw new Error(
      `commitment must be 32 bytes (got ${params.commitment.length})`
    );
  }
}

// src/utils/pda.ts
var import_web34 = require("@solana/web3.js");
function getShieldPoolPDAs(programId) {
  const pid = programId || CLOAK_PROGRAM_ID;
  const [pool] = import_web34.PublicKey.findProgramAddressSync(
    [Buffer.from("pool")],
    pid
  );
  const [commitments] = import_web34.PublicKey.findProgramAddressSync(
    [Buffer.from("commitments")],
    pid
  );
  const [rootsRing] = import_web34.PublicKey.findProgramAddressSync(
    [Buffer.from("roots_ring")],
    pid
  );
  const [nullifierShard] = import_web34.PublicKey.findProgramAddressSync(
    [Buffer.from("nullifier_shard")],
    pid
  );
  const [treasury] = import_web34.PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    pid
  );
  return {
    pool,
    commitments,
    rootsRing,
    nullifierShard,
    treasury
  };
}

// src/core/CloakSDK.ts
var CLOAK_PROGRAM_ID = new import_web35.PublicKey("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");
var CLOAK_API_URL = (
  // "http://localhost:80"; 
  "https://api.cloaklabz.xyz"
);
var CloakSDK = class {
  /**
  * Create a new Cloak SDK client
   *
   * @param config - Client configuration
   * 
   * @example
   * ```typescript
   * // Enhanced mode with v2.0 features (recommended)
   * const keys = generateCloakKeys();
   * const sdk = new CloakSDK({
   *   keypairBytes: keypair.secretKey,
   *   cloakKeys: keys,
   *   network: "devnet"
   * });
   * 
   * // Legacy mode (v1.0)
   * const sdk = new CloakSDK({
   *   keypairBytes: keypair.secretKey,
   *   network: "devnet"
   * });
   * ```
   */
  constructor(config) {
    this.keypair = import_web35.Keypair.fromSecretKey(config.keypairBytes);
    this.cloakKeys = config.cloakKeys;
    this.storage = config.storage || new MemoryStorageAdapter();
    const apiUrl = config.apiUrl || CLOAK_API_URL;
    this.indexer = new IndexerService(apiUrl);
    this.prover = new ProverService(apiUrl, 5 * 60 * 1e3);
    this.relay = new RelayService(apiUrl);
    this.depositRecovery = new DepositRecoveryService(this.indexer, apiUrl);
    if (!this.cloakKeys) {
      const storedKeys = this.storage.loadKeys();
      if (storedKeys && !(storedKeys instanceof Promise)) {
        this.cloakKeys = storedKeys;
      }
    }
    const { pool, commitments, rootsRing, nullifierShard, treasury } = getShieldPoolPDAs();
    this.config = {
      network: config.network || "devnet",
      keypairBytes: config.keypairBytes,
      cloakKeys: config.cloakKeys,
      apiUrl,
      poolAddress: pool,
      commitmentsAddress: commitments,
      rootsRingAddress: rootsRing,
      nullifierShardAddress: nullifierShard,
      treasuryAddress: treasury
    };
  }
  /**
   * Deposit SOL into the Cloak protocol
   *
   * Creates a new note (or uses a provided one), submits a deposit transaction,
   * and registers with the indexer.
   *
   * @param connection - Solana connection
   * @param payer - Payer wallet with sendTransaction method
   * @param amountOrNote - Amount in lamports OR an existing note to deposit
   * @param options - Optional configuration
   * @returns Deposit result with note and transaction info
   *
   * @example
   * ```typescript
   * // Generate and deposit in one step
   * const result = await client.deposit(
   *   connection,
   *   wallet,
   *   1_000_000_000,
   *   {
   *     onProgress: (status) => console.log(status)
   *   }
   * );
   *
   * // Or deposit a pre-generated note
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.deposit(connection, wallet, note);
   * ```
   */
  async deposit(connection, amountOrNote, options) {
    try {
      let note;
      if (typeof amountOrNote === "number") {
        options?.onProgress?.("Generating note...");
        note = generateNote(amountOrNote, this.config.network);
      } else {
        note = amountOrNote;
        if (note.depositSignature) {
          throw new Error("Note has already been deposited");
        }
      }
      options?.onProgress?.("Checking account balance...");
      const balance = await connection.getBalance(this.keypair.publicKey);
      const requiredAmount = note.amount + 5e3;
      if (balance < requiredAmount) {
        throw new Error(
          `Insufficient balance. Required: ${requiredAmount} lamports (${note.amount} + fees), Available: ${balance} lamports`
        );
      }
      options?.onProgress?.("Creating deposit transaction...");
      const commitmentBytes = hexToBytes(note.commitment);
      const depositIx = createDepositInstruction({
        programId: CLOAK_PROGRAM_ID,
        payer: this.keypair.publicKey,
        pool: this.config.poolAddress,
        commitments: this.config.commitmentsAddress,
        amount: note.amount,
        commitment: commitmentBytes
      });
      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
      const transaction = new import_web35.Transaction({
        feePayer: this.keypair.publicKey,
        blockhash,
        lastValidBlockHeight
      }).add(depositIx);
      if (!options?.skipPreflight) {
        options?.onProgress?.("Simulating transaction...");
        const simulation = await connection.simulateTransaction(transaction);
        if (simulation.value.err) {
          const logs = simulation.value.logs?.join("\n") || "No logs";
          throw new Error(
            `Transaction simulation failed: ${JSON.stringify(simulation.value.err)}
Logs:
${logs}`
          );
        }
      }
      options?.onProgress?.("Sending transaction...");
      const signature = await connection.sendTransaction(transaction, [this.keypair], {
        skipPreflight: options?.skipPreflight || false,
        preflightCommitment: "confirmed",
        maxRetries: 3
      });
      options?.onProgress?.("Confirming transaction...");
      const confirmation = await connection.confirmTransaction({
        signature,
        blockhash,
        lastValidBlockHeight
      });
      if (confirmation.value.err) {
        throw new Error(
          `Transaction failed: ${JSON.stringify(confirmation.value.err)}`
        );
      }
      const txDetails = await connection.getTransaction(signature, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0
      });
      const depositSlot = txDetails?.slot ?? 0;
      options?.onProgress?.("Registering with indexer...");
      const encryptedOutput = this.encodeNote(note, options?.recipientViewKey);
      let depositResponse = null;
      let retries = 0;
      const maxRetries = 3;
      const baseDelayMs = 1e3;
      while (retries <= maxRetries) {
        try {
          depositResponse = await this.indexer.submitDeposit({
            leafCommit: note.commitment,
            encryptedOutput,
            txSignature: signature,
            slot: depositSlot
          });
          break;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          if (errorMessage.includes("Merkle tree") && errorMessage.includes("inconsistent") && retries < maxRetries) {
            retries++;
            const delayMs = baseDelayMs * Math.pow(2, retries - 1);
            options?.onProgress?.(`Merkle tree inconsistency detected, retrying in ${delayMs}ms... (attempt ${retries}/${maxRetries})`);
            await new Promise((resolve) => setTimeout(resolve, delayMs));
            continue;
          }
          throw error;
        }
      }
      if (!depositResponse || !depositResponse.leafIndex || !depositResponse.root) {
        throw new Error("Failed to submit deposit: Indexer did not return leaf index and root");
      }
      const leafIndex = depositResponse.leafIndex;
      const root = depositResponse.root;
      options?.onProgress?.("Fetching Merkle proof...");
      const merkleProof = await this.indexer.getMerkleProof(leafIndex);
      const updatedNote = updateNoteWithDeposit(note, {
        signature,
        slot: depositSlot,
        leafIndex,
        root,
        merkleProof: {
          pathElements: merkleProof.pathElements,
          pathIndices: merkleProof.pathIndices
        }
      });
      options?.onProgress?.("Deposit complete!");
      return {
        note: updatedNote,
        signature,
        leafIndex,
        root
      };
    } catch (error) {
      throw this.wrapError(error, "Deposit failed");
    }
  }
  /**
   * Private transfer with up to 5 recipients
   *
   * Handles the complete private transfer flow:
   * 1. If note is not deposited, deposits it first and waits for confirmation
   * 2. Generates a zero-knowledge proof
   * 3. Submits the withdrawal via relay service to recipients
   *
   * This is the main method for performing private transfers - it handles everything!
   *
   * @param connection - Solana connection (required for deposit if not already deposited)
   * @param payer - Payer wallet (required for deposit if not already deposited)
   * @param note - Note to spend (can be deposited or not)
   * @param recipients - Array of 1-5 recipients with amounts
   * @param options - Optional configuration
   * @returns Transfer result with signature and outputs
   *
   * @example
   * ```typescript
   * // Create a note (not deposited yet)
   * const note = client.generateNote(1_000_000_000);
   *
   * // privateTransfer handles the full flow: deposit + withdraw
   * const result = await client.privateTransfer(
   *   connection,
   *   wallet,
   *   note,
   *   [
   *     { recipient: new PublicKey("..."), amount: 500_000_000 },
   *     { recipient: new PublicKey("..."), amount: 492_500_000 }
   *   ],
   *   {
   *     relayFeeBps: 50, // 0.5%
   *     onProgress: (status) => console.log(status),
   *     onProofProgress: (pct) => console.log(`Proof: ${pct}%`)
   *   }
   * );
   * console.log(`Success! TX: ${result.signature}`);
   * ```
   */
  async privateTransfer(connection, note, recipients, options) {
    if (!isWithdrawable(note)) {
      options?.onProgress?.("Note not deposited yet - depositing first...");
      const depositResult = await this.deposit(connection, note, {
        onProgress: options?.onProgress,
        skipPreflight: false
      });
      note = depositResult.note;
      options?.onProgress?.("Deposit complete - proceeding with private transfer...");
    }
    const protocolFee = note.amount - getDistributableAmount2(note.amount);
    const feeBps = Math.ceil(protocolFee * 1e4 / note.amount);
    const distributableAmount = getDistributableAmount2(note.amount);
    validateTransfers(recipients, distributableAmount);
    options?.onProgress?.("Fetching Merkle proof...");
    let merkleProof;
    let merkleRoot;
    if (note.merkleProof && note.root) {
      merkleProof = {
        pathElements: note.merkleProof.pathElements,
        pathIndices: note.merkleProof.pathIndices
      };
      merkleRoot = note.root;
    } else {
      merkleProof = await this.indexer.getMerkleProof(note.leafIndex);
      merkleRoot = merkleProof.root || (await this.indexer.getMerkleRoot()).root;
    }
    options?.onProgress?.("Computing cryptographic values...");
    const skSpendBytes = hexToBytes(note.sk_spend);
    const nullifierBytes = computeNullifier(skSpendBytes, note.leafIndex);
    const nullifierHex = bytesToHex(nullifierBytes);
    const outputsHashBytes = computeOutputsHash(recipients);
    const outputsHashHex = bytesToHex(outputsHashBytes);
    options?.onProgress?.("Generating zero-knowledge proof...");
    if (!note.leafIndex && note.leafIndex !== 0) {
      throw new Error("Note must have a leaf index (note must be deposited)");
    }
    if (!merkleProof.pathElements || merkleProof.pathElements.length === 0) {
      throw new Error("Merkle proof is invalid: missing path elements");
    }
    if (merkleProof.pathElements.length !== merkleProof.pathIndices.length) {
      throw new Error("Merkle proof is invalid: path elements and indices length mismatch");
    }
    for (let i = 0; i < merkleProof.pathIndices.length; i++) {
      const idx = merkleProof.pathIndices[i];
      if (idx !== 0 && idx !== 1) {
        throw new Error(`Merkle proof path index at position ${i} must be 0 or 1, got ${idx}`);
      }
    }
    if (!isValidHex(note.r, 32)) {
      throw new Error("Note r must be 64 hex characters (32 bytes)");
    }
    if (!isValidHex(note.sk_spend, 32)) {
      throw new Error("Note sk_spend must be 64 hex characters (32 bytes)");
    }
    if (!isValidHex(merkleRoot, 32)) {
      throw new Error("Merkle root must be 64 hex characters (32 bytes)");
    }
    for (let i = 0; i < merkleProof.pathElements.length; i++) {
      const element = merkleProof.pathElements[i];
      if (typeof element !== "string" || !isValidHex(element, 32)) {
        throw new Error(`Merkle proof path element at position ${i} must be 64 hex characters (32 bytes)`);
      }
    }
    const proofInputs = {
      privateInputs: {
        amount: note.amount,
        r: note.r,
        sk_spend: note.sk_spend,
        leaf_index: note.leafIndex,
        merkle_path: {
          path_elements: merkleProof.pathElements,
          path_indices: merkleProof.pathIndices
        }
      },
      publicInputs: {
        root: merkleRoot,
        nf: nullifierHex,
        outputs_hash: outputsHashHex,
        amount: note.amount
      },
      outputs: recipients.map((r) => ({
        address: r.recipient.toBase58(),
        amount: r.amount
      }))
    };
    const proofResult = await this.prover.generateProof(proofInputs, {
      onProgress: options?.onProofProgress,
      onStart: () => options?.onProgress?.("Starting proof generation..."),
      onSuccess: () => options?.onProgress?.("Proof generated successfully"),
      onError: (error) => {
        console.error("[CloakSDK] Proof generation error:", error);
        options?.onProgress?.(`Proof generation error: ${error}`);
      }
    });
    if (!proofResult.success || !proofResult.proof || !proofResult.publicInputs) {
      let errorMessage = proofResult.error || "Proof generation failed";
      if (errorMessage.startsWith("Proof generation failed: ")) {
        errorMessage = errorMessage.substring("Proof generation failed: ".length);
      }
      errorMessage += `
Note details: leafIndex=${note.leafIndex}, root=${merkleRoot.slice(0, 16)}..., nullifier=${nullifierHex.slice(0, 16)}...`;
      throw new Error(errorMessage);
    }
    options?.onProgress?.("Submitting to relay service...");
    const signature = await this.relay.submitWithdraw(
      {
        proof: proofResult.proof,
        publicInputs: {
          root: merkleRoot,
          nf: nullifierHex,
          outputs_hash: outputsHashHex,
          amount: note.amount
        },
        outputs: recipients.map((r) => ({
          recipient: r.recipient.toBase58(),
          amount: r.amount
        })),
        feeBps
        // Use calculated protocol fee BPS
      },
      options?.onProgress
    );
    options?.onProgress?.("Transfer complete!");
    return {
      signature,
      outputs: recipients.map((r) => ({
        recipient: r.recipient.toBase58(),
        amount: r.amount
      })),
      nullifier: nullifierHex,
      root: merkleRoot
    };
  }
  /**
   * Withdraw to a single recipient
   *
   * Convenience method for withdrawing to one address.
   * Handles the complete flow: deposits if needed, then withdraws.
   *
   * @param connection - Solana connection
   * @param payer - Payer wallet
   * @param note - Note to spend
   * @param recipient - Recipient address
   * @param options - Optional configuration
   * @returns Transfer result
   *
   * @example
   * ```typescript
   * const note = client.generateNote(1_000_000_000);
   * const result = await client.withdraw(
   *   connection,
   *   wallet,
   *   note,
   *   new PublicKey("..."),
   *   { withdrawAll: true }
   * );
   * ```
   */
  async withdraw(connection, note, recipient, options) {
    const withdrawAll = options?.withdrawAll ?? true;
    const amount = withdrawAll ? getDistributableAmount2(note.amount) : options?.amount || note.amount;
    if (!withdrawAll && !options?.amount) {
      throw new Error("Must specify amount or set withdrawAll: true");
    }
    return this.privateTransfer(
      connection,
      note,
      [{ recipient, amount }],
      options
    );
  }
  /**
   * Generate a new note without depositing
   *
   * @param amountLamports - Amount for the note
   * @param useWalletKeys - Whether to use wallet keys (v2.0 recommended)
   * @returns New note (not yet deposited)
   */
  generateNote(amountLamports, useWalletKeys = false) {
    if (useWalletKeys && this.cloakKeys) {
      return generateNoteFromWallet(amountLamports, this.cloakKeys, this.config.network);
    } else if (useWalletKeys) {
      const keys = generateCloakKeys();
      this.cloakKeys = keys;
      const result = this.storage.saveKeys(keys);
      if (result instanceof Promise) {
        result.catch(() => {
        });
      }
      return generateNoteFromWallet(amountLamports, keys, this.config.network);
    }
    return generateNote(amountLamports, this.config.network);
  }
  /**
   * Parse a note from JSON string
   *
   * @param jsonString - JSON representation
   * @returns Parsed note
   */
  parseNote(jsonString) {
    return parseNote2(jsonString);
  }
  /**
   * Export a note to JSON string
   *
   * @param note - Note to export
   * @param pretty - Format with indentation
   * @returns JSON string
   */
  exportNote(note, pretty = false) {
    return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
  }
  /**
   * Check if a note is ready for withdrawal
   *
   * @param note - Note to check
   * @returns True if withdrawable
   */
  isWithdrawable(note) {
    return isWithdrawable(note);
  }
  /**
   * Get Merkle proof for a leaf index
   *
   * @param leafIndex - Leaf index in tree
   * @returns Merkle proof
   */
  async getMerkleProof(leafIndex) {
    return this.indexer.getMerkleProof(leafIndex);
  }
  /**
   * Get current Merkle root
   *
   * @returns Current root hash
   */
  async getCurrentRoot() {
    const response = await this.indexer.getMerkleRoot();
    return response.root;
  }
  /**
   * Get transaction status from relay service
   *
   * @param requestId - Request ID from previous submission
   * @returns Current status
   */
  async getTransactionStatus(requestId) {
    return this.relay.getStatus(requestId);
  }
  /**
   * Recover a deposit that completed on-chain but failed to register
   * 
   * Use this when a deposit transaction succeeded but the browser crashed
   * or lost connection before the indexer registration completed.
   * 
   * @param signature - Transaction signature
   * @param commitment - Note commitment hash
   * @param note - Optional: The full note if available
   * @returns Recovery result with updated note
   * 
   * @example
   * ```typescript
   * const result = await sdk.recoverDeposit({
   *   signature: "5Kn4...",
   *   commitment: "abc123...",
   *   note: myNote // optional if you have it
   * });
   * 
   * if (result.success) {
   *   console.log(`Recovered! Leaf index: ${result.leafIndex}`);
   * }
   * ```
   */
  async recoverDeposit(options) {
    return this.depositRecovery.recoverDeposit(options);
  }
  /**
   * Load all notes from storage
   * 
   * @returns Array of saved notes
   */
  async loadNotes() {
    const notes = this.storage.loadAllNotes();
    return Array.isArray(notes) ? notes : await notes;
  }
  /**
   * Save a note to storage
   * 
   * @param note - Note to save
   */
  async saveNote(note) {
    const result = this.storage.saveNote(note);
    if (result instanceof Promise) {
      await result;
    }
  }
  /**
   * Find a note by its commitment
   * 
   * @param commitment - Commitment hash
   * @returns Note if found
   */
  async findNote(commitment) {
    const notes = await this.loadNotes();
    return findNoteByCommitment(notes, commitment);
  }
  /**
   * Import wallet keys from JSON
   * 
   * @param keysJson - JSON string containing keys
   */
  async importWalletKeys(keysJson) {
    const keys = importWalletKeys(keysJson);
    this.cloakKeys = keys;
    const result = this.storage.saveKeys(keys);
    if (result instanceof Promise) {
      await result;
    }
  }
  /**
   * Export wallet keys to JSON
   * 
   * WARNING: This exports secret keys! Store securely.
   * 
   * @returns JSON string with keys
   */
  exportWalletKeys() {
    if (!this.cloakKeys) {
      throw new CloakError("No wallet keys available", "wallet", false);
    }
    return exportWalletKeys(this.cloakKeys);
  }
  /**
   * Get the configuration
   */
  getConfig() {
    return { ...this.config };
  }
  /**
   * Encode note data for indexer storage
   * 
   * Enhanced version that supports encrypted outputs for v2.0 scanning
   */
  encodeNote(note, recipientViewKey) {
    if (recipientViewKey) {
      return prepareEncryptedOutputForRecipient(note, recipientViewKey);
    }
    if (this.cloakKeys) {
      return prepareEncryptedOutput(note, this.cloakKeys);
    }
    return encodeNoteSimple(note);
  }
  /**
   * Scan blockchain for notes belonging to this wallet (v2.0 feature)
   * 
   * Requires Cloak keys to be configured in the SDK.
   * Fetches encrypted outputs from the indexer and decrypts notes
   * that belong to this wallet.
   * 
   * @param options - Scanning options
   * @returns Array of discovered notes with metadata
   * 
   * @example
   * ```typescript
   * const notes = await sdk.scanNotes({
   *   onProgress: (current, total) => {
   *     console.log(`Scanning: ${current}/${total}`);
   *   }
   * });
   * 
   * console.log(`Found ${notes.length} notes!`);
   * const totalBalance = notes.reduce((sum, n) => sum + n.amount, 0);
   * ```
   */
  async scanNotes(options) {
    if (!this.cloakKeys) {
      throw new CloakError(
        "Note scanning requires Cloak keys. Initialize SDK with: cloakKeys: generateCloakKeys()",
        "validation",
        false
      );
    }
    const startIndex = options?.startIndex ?? 0;
    const batchSize = options?.batchSize ?? 100;
    const { next_index: totalNotes } = await this.indexer.getMerkleRoot();
    const endIndex = options?.endIndex ?? (totalNotes > 0 ? totalNotes - 1 : 0);
    if (totalNotes === 0 || endIndex < startIndex) {
      return [];
    }
    const allEncryptedOutputs = [];
    for (let start = startIndex; start <= endIndex; start += batchSize) {
      const end = Math.min(start + batchSize - 1, endIndex);
      options?.onProgress?.(start, totalNotes);
      const { notes } = await this.indexer.getNotesRange(start, end, batchSize);
      allEncryptedOutputs.push(...notes);
    }
    options?.onProgress?.(totalNotes, totalNotes);
    const foundNoteData = scanNotesForWallet(
      allEncryptedOutputs,
      this.cloakKeys.view
    );
    const scannedNotes = foundNoteData.map((noteData) => ({
      version: "2.0",
      amount: noteData.amount,
      commitment: noteData.commitment,
      sk_spend: noteData.sk_spend,
      r: noteData.r,
      timestamp: Date.now(),
      network: this.config.network || "devnet",
      scannedAt: Date.now()
    }));
    return scannedNotes;
  }
  /**
   * Wrap errors with better categorization and user-friendly messages
   * 
   * @private
   */
  wrapError(error, context) {
    if (error instanceof CloakError) {
      return error;
    }
    const errorMessage = error instanceof Error ? error.message : String(error);
    if (errorMessage.includes("duplicate key") || errorMessage.includes("already deposited")) {
      return new CloakError(
        "This note was already deposited. The transaction succeeded but the indexer has it recorded. Generate a new note or scan for existing notes.",
        "indexer",
        false,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("insufficient funds") || errorMessage.includes("insufficient lamports")) {
      return new CloakError(
        "Insufficient funds for this transaction. Please check your wallet balance.",
        "wallet",
        false,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("Merkle tree") && errorMessage.includes("inconsistent")) {
      return new CloakError(
        "Indexer is temporarily unavailable. Please try again in a moment.",
        "indexer",
        true,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("timeout") || errorMessage.includes("timed out")) {
      return new CloakError(
        "Network timeout. Please check your connection and try again.",
        "network",
        true,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("not connected") || errorMessage.includes("wallet")) {
      return new CloakError(
        "Wallet not connected. Please connect your wallet first.",
        "wallet",
        false,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("proof") && (errorMessage.includes("failed") || errorMessage.includes("error"))) {
      return new CloakError(
        "Zero-knowledge proof generation failed. This is usually temporary - please try again.",
        "prover",
        true,
        error instanceof Error ? error : void 0
      );
    }
    if (errorMessage.includes("relay") || errorMessage.includes("withdraw")) {
      return new CloakError(
        "Relay service error. Please try again later.",
        "relay",
        true,
        error instanceof Error ? error : void 0
      );
    }
    return new CloakError(
      `${context}: ${errorMessage}`,
      "network",
      false,
      error instanceof Error ? error : void 0
    );
  }
};

// src/core/note.ts
function serializeNote(note, pretty = false) {
  validateNote(note);
  return pretty ? JSON.stringify(note, null, 2) : JSON.stringify(note);
}
function downloadNote(note, filename) {
  const g = globalThis;
  const doc = g?.document;
  const URL_ = g?.URL;
  const Blob_ = g?.Blob;
  if (!doc || !URL_ || !Blob_) {
    throw new Error("downloadNote is only available in browser environments");
  }
  const json = serializeNote(note, true);
  const blob = new Blob_([json], { type: "application/json" });
  const url = URL_.createObjectURL(blob);
  const defaultFilename = `cloak-note-${note.commitment.slice(0, 8)}.json`;
  const link = doc.createElement("a");
  link.href = url;
  link.download = filename || defaultFilename;
  link.click();
  URL_.revokeObjectURL(url);
}
async function copyNoteToClipboard(note) {
  const g = globalThis;
  const nav = g?.navigator;
  if (!nav || !nav.clipboard) {
    throw new Error("Clipboard API not available");
  }
  const json = serializeNote(note, true);
  await nav.clipboard.writeText(json);
}

// src/utils/errors.ts
var PROGRAM_ERRORS = {
  // Nullifier errors
  "NullifierAlreadyUsed": "This note has already been withdrawn. Each note can only be spent once.",
  "0x1770": "This note has already been withdrawn.",
  // Proof verification errors
  "ProofVerificationFailed": "Zero-knowledge proof verification failed. Please try again.",
  "0x1771": "Invalid proof. Please regenerate and try again.",
  "InvalidProof": "The provided proof is invalid.",
  // Root errors
  "RootNotFound": "The Merkle root is outdated or invalid. Please refresh and try again.",
  "InvalidRoot": "Invalid Merkle root. The tree may have been updated.",
  "0x1772": "Merkle root not found in history.",
  // Amount errors
  "InvalidAmount": "Invalid amount. Please check your input.",
  "InsufficientFunds": "Insufficient funds for this transaction.",
  "AmountMismatch": "Output amounts don't match the note amount.",
  // Fee errors
  "InvalidFee": "Invalid fee calculation. Please try again.",
  "FeeTooHigh": "Fee exceeds maximum allowed.",
  // Output errors
  "InvalidOutputs": "Invalid recipient configuration.",
  "TooManyOutputs": "Maximum 5 recipients allowed per transaction.",
  "OutputHashMismatch": "Output hash doesn't match proof.",
  // General errors
  "Unauthorized": "You don't have permission to perform this action.",
  "AccountNotFound": "Required account not found.",
  "InvalidInstruction": "Invalid instruction data."
};
function parseTransactionError(error) {
  if (!error) return "An unknown error occurred";
  const errorStr = typeof error === "string" ? error : error.message || error.toString();
  const hexMatch = errorStr.match(/0x[0-9a-f]{4}/i);
  if (hexMatch) {
    const errorCode = hexMatch[0];
    if (PROGRAM_ERRORS[errorCode]) {
      return PROGRAM_ERRORS[errorCode];
    }
  }
  for (const [key, message] of Object.entries(PROGRAM_ERRORS)) {
    if (errorStr.includes(key)) {
      return message;
    }
  }
  if (errorStr.includes("insufficient funds") || errorStr.includes("insufficient lamports")) {
    return "Insufficient SOL balance. Please add funds to your wallet.";
  }
  if (errorStr.includes("blockhash not found")) {
    return "Transaction expired. Please try again.";
  }
  if (errorStr.includes("already been processed")) {
    return "This transaction has already been processed.";
  }
  if (errorStr.includes("signature verification failed")) {
    return "Transaction signature verification failed. Please try again.";
  }
  if (errorStr.includes("account does not exist")) {
    return "Required account not found. The program may need to be initialized.";
  }
  if (errorStr.includes("fetch") || errorStr.includes("network")) {
    return "Network error. Please check your connection and try again.";
  }
  if (errorStr.includes("timeout")) {
    return "Request timed out. Please try again.";
  }
  if (errorStr.includes("relay") || errorStr.includes("withdraw")) {
    if (errorStr.includes("in progress")) {
      return "A withdrawal is already in progress. Please wait for it to complete.";
    }
    if (errorStr.includes("rate limit")) {
      return "Too many requests. Please wait a moment and try again.";
    }
    return "Relay service error. Please try again later.";
  }
  if (errorStr.includes("proof") && errorStr.includes("generation")) {
    return "Failed to generate zero-knowledge proof. Please try again.";
  }
  if (errorStr.includes("indexer") || errorStr.includes("merkle")) {
    if (errorStr.includes("inconsistent")) {
      return "The indexer is temporarily unavailable. Please try again in a moment.";
    }
    if (errorStr.includes("not found")) {
      return "Note not found in the indexer. It may not be confirmed yet.";
    }
    return "Indexer service error. Please try again later.";
  }
  let cleanError = errorStr.replace(/Error:\s*/gi, "").replace(/\s+at\s+.*$/g, "").replace(/\[.*?\]/g, "").trim();
  if (cleanError.length > 200) {
    cleanError = cleanError.substring(0, 197) + "...";
  }
  return cleanError || "Transaction failed. Please try again.";
}
function createCloakError(error, _context) {
  if (error instanceof CloakError) {
    return error;
  }
  const errorMessage = error instanceof Error ? error.message : String(error);
  const userMessage = parseTransactionError(error);
  let category = "network";
  let retryable = false;
  if (errorMessage.includes("insufficient") || errorMessage.includes("balance")) {
    category = "wallet";
    retryable = false;
  } else if (errorMessage.includes("proof")) {
    category = "prover";
    retryable = true;
  } else if (errorMessage.includes("indexer") || errorMessage.includes("merkle")) {
    category = "indexer";
    retryable = errorMessage.includes("inconsistent") || errorMessage.includes("temporary");
  } else if (errorMessage.includes("relay")) {
    category = "relay";
    retryable = true;
  } else if (errorMessage.includes("timeout") || errorMessage.includes("network")) {
    category = "network";
    retryable = true;
  } else if (errorMessage.includes("validation") || errorMessage.includes("invalid")) {
    category = "validation";
    retryable = false;
  }
  return new CloakError(
    userMessage,
    category,
    retryable,
    error instanceof Error ? error : void 0
  );
}
function formatErrorForLogging(error) {
  if (error instanceof CloakError) {
    return `[${error.category}] ${error.message}${error.retryable ? " (retryable)" : ""}`;
  }
  if (error instanceof Error) {
    return `${error.name}: ${error.message}`;
  }
  return String(error);
}

// src/helpers/wallet-integration.ts
var import_web36 = require("@solana/web3.js");
function validateWalletConnected(wallet) {
  if (wallet instanceof import_web36.Keypair) {
    return;
  }
  if (!wallet.publicKey) {
    throw new CloakError(
      "Wallet not connected. Please connect your wallet first.",
      "wallet",
      false
    );
  }
}
function getPublicKey(wallet) {
  if (wallet instanceof import_web36.Keypair) {
    return wallet.publicKey;
  }
  if (!wallet.publicKey) {
    throw new CloakError(
      "Wallet not connected",
      "wallet",
      false
    );
  }
  return wallet.publicKey;
}
async function sendTransaction(transaction, wallet, connection, options) {
  if (wallet instanceof import_web36.Keypair) {
    return await connection.sendTransaction(transaction, [wallet], options);
  }
  if (wallet.sendTransaction) {
    return await wallet.sendTransaction(transaction, connection, options);
  } else if (wallet.signTransaction) {
    const signed = await wallet.signTransaction(transaction);
    return await connection.sendRawTransaction(signed.serialize(), options);
  } else {
    throw new CloakError(
      "Wallet does not support transaction signing",
      "wallet",
      false
    );
  }
}
async function signTransaction(transaction, wallet) {
  if (wallet instanceof import_web36.Keypair) {
    transaction.sign(wallet);
    return transaction;
  }
  if (!wallet.signTransaction) {
    throw new CloakError(
      "Wallet does not support transaction signing",
      "wallet",
      false
    );
  }
  return await wallet.signTransaction(transaction);
}
function keypairToAdapter(keypair) {
  return {
    publicKey: keypair.publicKey,
    signTransaction: async (tx) => {
      tx.sign(keypair);
      return tx;
    },
    signAllTransactions: async (txs) => {
      txs.forEach((tx) => tx.sign(keypair));
      return txs;
    }
  };
}

// src/index.ts
var VERSION = "0.1.0";
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  CloakError,
  CloakSDK,
  DepositRecoveryService,
  FIXED_FEE_LAMPORTS,
  IndexerService,
  LAMPORTS_PER_SOL,
  LocalStorageAdapter,
  MemoryStorageAdapter,
  ProverService,
  RelayService,
  VARIABLE_FEE_RATE,
  VERSION,
  bytesToHex,
  calculateFee,
  calculateRelayFee,
  computeNullifier,
  computeOutputsHash,
  copyNoteToClipboard,
  createCloakError,
  createDepositInstruction,
  deriveSpendKey,
  deriveViewKey,
  detectNetworkFromRpcUrl,
  downloadNote,
  encodeNoteSimple,
  encryptNoteForRecipient,
  exportKeys,
  exportNote,
  exportWalletKeys,
  filterNotesByNetwork,
  filterWithdrawableNotes,
  findNoteByCommitment,
  formatAmount,
  formatErrorForLogging,
  generateCloakKeys,
  generateCommitment,
  generateMasterSeed,
  generateNote,
  generateNoteFromWallet,
  getAddressExplorerUrl,
  getDistributableAmount,
  getExplorerUrl,
  getPublicKey,
  getPublicViewKey,
  getRecipientAmount,
  getRpcUrlForNetwork,
  getViewKey,
  hexToBytes,
  importKeys,
  importWalletKeys,
  isValidHex,
  isValidRpcUrl,
  isValidSolanaAddress,
  isWithdrawable,
  keypairToAdapter,
  parseAmount,
  parseNote,
  parseTransactionError,
  prepareEncryptedOutput,
  prepareEncryptedOutputForRecipient,
  randomBytes,
  scanNotesForWallet,
  sendTransaction,
  serializeNote,
  signTransaction,
  tryDecryptNote,
  updateNoteWithDeposit,
  validateDepositParams,
  validateNote,
  validateOutputsSum,
  validateTransfers,
  validateWalletConnected,
  validateWithdrawableNote
});
