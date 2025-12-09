#!/bin/bash
# Script para redeploy do shield-pool na devnet mantendo o mesmo program ID

set -e

cd "$(dirname "$0")/.."

PROGRAM_ID="c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp"
PROGRAM_KEYPAIR="${PROGRAM_ID}.json"
AUTHORITY="mgfSqUe1qaaUjeEzuLUyDUx5Rk4fkgePB5NtLnS3Vxa"

echo "🚀 Redeploy do Shield Pool na Devnet"
echo "======================================"
echo "Program ID: ${PROGRAM_ID}"
echo "Authority: ${AUTHORITY}"
echo ""

# Verificar se o binário existe
if [ ! -f "target/deploy/shield_pool.so" ]; then
    echo "❌ Binário não encontrado: target/deploy/shield_pool.so"
    echo "   Por favor, recompile o programa primeiro:"
    echo "   cd programs/shield-pool && cargo build-sbf"
    exit 1
fi

# Verificar tamanho do binário
BINARY_SIZE=$(stat -c%s target/deploy/shield_pool.so)
echo "📦 Tamanho do binário: ${BINARY_SIZE} bytes"

# Verificar tamanho atual do programa na devnet
echo ""
echo "🔍 Verificando programa atual na devnet..."
CURRENT_SIZE=$(solana program show ${PROGRAM_ID} --url devnet 2>&1 | grep "Data Length" | awk '{print $3}' || echo "0")
if [ "$CURRENT_SIZE" != "0" ]; then
    echo "   Tamanho atual: ${CURRENT_SIZE} bytes"
    
    # Se o novo binário for maior, precisamos estender
    if [ ${BINARY_SIZE} -gt ${CURRENT_SIZE} ]; then
        EXTEND_BYTES=$((BINARY_SIZE - CURRENT_SIZE + 10240)) # Adiciona 10KB de margem
        echo ""
        echo "📏 Estendendo programa em ${EXTEND_BYTES} bytes..."
        solana program extend ${PROGRAM_ID} ${EXTEND_BYTES} --url devnet --keypair ${PROGRAM_KEYPAIR} || {
            echo "⚠️  Erro ao estender. Tentando com authority keypair..."
            # Tentar encontrar o keypair do authority
            if [ -f "admin-keypair.json" ]; then
                ADMIN_PUBKEY=$(solana-keygen pubkey admin-keypair.json)
                if [ "${ADMIN_PUBKEY}" = "${AUTHORITY}" ]; then
                    solana program extend ${PROGRAM_ID} ${EXTEND_BYTES} --url devnet --keypair admin-keypair.json
                else
                    echo "❌ admin-keypair.json não é o authority. Por favor, forneça o keypair correto."
                    exit 1
                fi
            else
                echo "❌ Keypair do authority não encontrado. Por favor, forneça o keypair para: ${AUTHORITY}"
                exit 1
            fi
        }
    fi
else
    echo "⚠️  Não foi possível obter o tamanho atual do programa"
fi

# Fazer o upgrade
echo ""
echo "⬆️  Fazendo upgrade do programa..."
solana program deploy target/deploy/shield_pool.so \
    --program-id ${PROGRAM_KEYPAIR} \
    --url devnet \
    --upgrade-authority ${AUTHORITY} || {
    echo "⚠️  Erro no deploy. Tentando com keypair do authority..."
    if [ -f "admin-keypair.json" ]; then
        ADMIN_PUBKEY=$(solana-keygen pubkey admin-keypair.json)
        if [ "${ADMIN_PUBKEY}" = "${AUTHORITY}" ]; then
            solana program deploy target/deploy/shield_pool.so \
                --program-id ${PROGRAM_KEYPAIR} \
                --url devnet \
                --keypair admin-keypair.json
        else
            echo "❌ admin-keypair.json não é o authority. Por favor, forneça o keypair correto."
            exit 1
        fi
    else
        echo "❌ Keypair do authority não encontrado. Por favor, forneça o keypair para: ${AUTHORITY}"
        exit 1
    fi
}

echo ""
echo "✅ Deploy concluído!"
echo "   Verifique o programa: solana program show ${PROGRAM_ID} --url devnet"

