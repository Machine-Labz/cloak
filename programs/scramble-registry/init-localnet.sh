#!/bin/bash
set -e

echo "=== Initializing Scramble Registry on Localnet ==="
echo ""

PROGRAM_ID="AWNvgBjSBpEPQRTWk63CwHAfiAqMkW5DSQ3Py8sz7C3g"
ADMIN=$(solana address)

echo "Program ID: $PROGRAM_ID"
echo "Admin: $ADMIN"
echo ""

# Note: You'll need to implement the actual initialization instruction
# This is a placeholder for the initialization transaction

echo "TODO: Implement initialization transaction"
echo ""
echo "The registry needs to be initialized with:"
echo "  - Admin pubkey: $ADMIN"
echo "  - Initial difficulty"
echo "  - Reveal window (slots)"
echo "  - Claim window (slots)"
echo "  - Max k"
echo ""
echo "Once implemented, this script will:"
echo "  1. Derive registry PDA"
echo "  2. Build initialize_registry instruction"
echo "  3. Submit transaction"
echo "  4. Confirm initialization"
