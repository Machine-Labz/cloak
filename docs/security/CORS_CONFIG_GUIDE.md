# 🌐 CORS Configuration Guide

## Environment Variable

**Name:** `CORS_ORIGINS`

**Where to configure:**
- `services/indexer/.env`
- `services/relay/.env`

---

## How It Works

### Format

**Separate multiple origins with commas** (with or without spaces, both work):

```bash
CORS_ORIGINS=https://site1.com,https://site2.com,https://site3.com
```

**Or with spaces** (code automatically does `trim()`):

```bash
CORS_ORIGINS=https://site1.com, https://site2.com, https://site3.com
```

---

## Practical Examples

### Example 1: Local Development (Allow Everything)

```bash
# Option 1: Don't define (uses automatic default)
# If NODE_ENV=development, allows all origins automatically

# Option 2: Define explicitly
CORS_ORIGINS=*
```

**Result:** Any origin can make requests ✅

---

### Example 2: Production (Only Your Domains)

```bash
# Only your production domains
CORS_ORIGINS=https://cloak.network,https://app.cloak.network
```

**Result:** Only these 2 domains can make requests ✅

---

### Example 3: Multiple Domains

```bash
# Main site + subdomains + staging
CORS_ORIGINS=https://cloak.network,https://app.cloak.network,https://wallet.cloak.network,https://staging.cloak.network
```

**Result:** All these domains can make requests ✅

---

### Example 4: With Ports (Development)

```bash
# Localhost with different ports
CORS_ORIGINS=http://localhost:3000,http://localhost:3001,http://127.0.0.1:3000
```

**Result:** Only these local addresses can make requests ✅

---

## Default Behavior (If Not Defined)

If you **don't define** `CORS_ORIGINS`, the system uses defaults based on `NODE_ENV`:

### If `NODE_ENV=development`:
```bash
# Don't need to define CORS_ORIGINS
# System automatically uses: *
```
**Result:** Allows all origins (development)

### If `NODE_ENV=production`:
```bash
# Don't need to define CORS_ORIGINS
# System automatically uses:
# - https://cloak.network
# - https://app.cloak.network
```
**Result:** Only these 2 default domains

---

## Important Rules

### ✅ Valid URLs

- ✅ Must include protocol (`http://` or `https://`)
- ✅ Can include port (`http://localhost:3000`)
- ✅ Can have subdomains (`https://app.cloak.network`)
- ✅ Can have path (but usually not necessary)

### ❌ Invalid URLs

- ❌ Without protocol: `cloak.network` (error!)
- ❌ With trailing slash: `https://cloak.network/` (may not work)
- ❌ With wildcards: `*.cloak.network` (not supported)

---

## Complete Configuration

### For Indexer (`services/indexer/.env`):

```bash
# CORS Origins (optional - if not defined, uses default)
CORS_ORIGINS=https://cloak.network,https://app.cloak.network

# Other variables...
DATABASE_URL=postgres://...
SOLANA_RPC_URL=http://...
NODE_ENV=production
```

### For Relay (`services/relay/.env`):

```bash
# CORS Origins (optional - if not defined, uses default)
CORS_ORIGINS=https://cloak.network,https://app.cloak.network

# Other variables...
DATABASE_URL=postgres://...
SOLANA_RPC_URL=http://...
```

---

## Verification

### How to test if it's working:

**1. Test with allowed origin:**
```bash
curl -H "Origin: https://cloak.network" \
     -H "Access-Control-Request-Method: POST" \
     -X OPTIONS \
     http://localhost:3001/api/v1/merkle/root
```

**2. Test with non-allowed origin:**
```bash
curl -H "Origin: https://evil.com" \
     -H "Access-Control-Request-Method: POST" \
     -X OPTIONS \
     http://localhost:3001/api/v1/merkle/root
```

**3. In browser:**
- Open DevTools → Network
- Make a request
- Check the `Access-Control-Allow-Origin` header in the response

---

## Troubleshooting

### Problem: "CORS policy blocked"

**Solution:**
1. Check if the origin is in the `CORS_ORIGINS` list
2. Check if you included the protocol (`https://` or `http://`)
3. Check if there's no trailing slash
4. Restart the service after changing `.env`

### Problem: "Credentials not allowed"

**Solution:**
- If using `*` (all origins), credentials are not allowed
- Use a specific list of origins to allow credentials

---

## Quick Summary

```bash
# Variable
CORS_ORIGINS

# Format
origin1,origin2,origin3

# Example
CORS_ORIGINS=https://site1.com,https://site2.com

# Where
services/indexer/.env
services/relay/.env

# Default if not defined
- Dev: * (all)
- Prod: https://cloak.network,https://app.cloak.network
```

---

## Complete `.env` Example

```bash
# services/indexer/.env

# CORS Configuration
CORS_ORIGINS=https://cloak.network,https://app.cloak.network,https://wallet.cloak.network

# Environment
NODE_ENV=production

# Database
DATABASE_URL=postgres://user:pass@localhost:5432/indexer

# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
CLOAK_PROGRAM_ID=...

# Other configurations...
```

Done! 🎯
