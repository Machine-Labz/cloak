#!/bin/bash
# Script para gerar o VKEY hash do circuito SP1

set -e

cd "$(dirname "$0")/.."

echo "🔍 Procurando ELF do guest..."
ELF_PATH=""

for path in \
    "target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest" \
    "packages/zk-guest-sp1/guest/target/riscv32im-succinct-zkvm-elf/release/zk-guest-sp1-guest" \
    "target/release/zk-guest-sp1-guest"
do
    if [ -f "$path" ]; then
        ELF_PATH="$path"
        echo "✅ ELF encontrado: $ELF_PATH"
        break
    fi
done

if [ -z "$ELF_PATH" ]; then
    echo "❌ ELF não encontrado. Compile o guest primeiro:"
    echo "   cargo build -p zk-guest-sp1-guest --release"
    exit 1
fi

echo ""
echo "🔑 Gerando VKEY hash..."
echo "   (Isso pode levar alguns segundos...)"

# Tenta usar o binário get_vkey_hash se disponível
if cargo run --package zk-guest-sp1-host --bin get_vkey_hash --release 2>/dev/null | grep "SP1 Withdraw Circuit VKey Hash:"; then
    exit 0
fi

echo "⚠️  Não foi possível executar via cargo. Verifique os logs do indexer:"
echo "   O indexer imprime o VKEY hash quando inicia."
echo ""
echo "   Procure por: 'SP1 VKEY hash:' ou 'VKEY hash:' nos logs do indexer"

