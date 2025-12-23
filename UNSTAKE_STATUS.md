# Unstake - Status e Problemas Identificados

## ✅ Implementações Completas

### 1. **ZK Circuit** 
- ✅ Circuit modificado para aceitar `unstake_params`
- ✅ Verifica que `commitment` (deposit) está correto
- ✅ Skips Merkle path & nullifier verification (unstake é deposit, não withdraw)

### 2. **Indexer/TEE**
- ✅ `tee_handlers.rs` aceita `unstake_params`
- ✅ `sp1_tee_client.rs` passa `unstake_params` para o guest program
- ✅ Prova ZK é gerada corretamente

### 3. **Frontend - Parte 1 (ZK Proof)**
- ✅ UI com seletor de stake accounts (só mostra `inactive`)
- ✅ Gera nova commitment (`r`, `sk_spend`) para o deposit no pool
- ✅ Calcula `stake_account_hash`, `outputs_hash`
- ✅ Fee calculation (0.5% variable)
- ✅ Gera prova ZK via TEE artifact flow
- ✅ Envia prova em base64 para relay
- ✅ Salva novo `CloakNote` em localStorage

### 4. **Relay - Parte 1 (API)**
- ✅ Endpoint `/unstake` criado e roteado
- ✅ Aceita payload com `proof`, `public_inputs`, `unstake` config
- ✅ Cria job no banco de dados
- ✅ Adiciona campo `partially_signed_tx` (opcional)

---

## 🚨 PROBLEMAS CRÍTICOS

### Problema 1: **Falta Assinatura do Stake Authority**

#### Descrição
A transação `UnstakeToPool` requer **2 assinaturas**:
1. **stake_authority** (usuário) - autoriza retirada de fundos do stake account
2. **fee_payer** (relay) - paga as transaction fees

**Atualmente**, o relay só assina com `fee_payer`, então a transação **falha on-chain** com:
```
Error: Transaction signature verification failure
```

#### Por que falha?
O Solana Stake Program exige que o `stake_authority` assine qualquer transação que retira fundos de um stake account. Sem essa assinatura, a transação é rejeitada.

#### Solução
Implementar **fluxo de 2 fases** (igual ao Stake):

**Fase 1 - Frontend:**
```typescript
// 1. Criar transação UnstakeToPool
const unstakeTx = await buildUnstakeToPoolTransaction({
  proof: proofBytes,
  publicInputs: public104,
  stakeAccount: stakeAccountPubkey,
  stakeAuthority: publicKey, // User's wallet
  programId,
  poolPda,
  rootsRingPda,
  feePayer: relayFeePayer,
  recentBlockhash,
});

// 2. Usuário assina como stake_authority
await sendTransaction(unstakeTx, connection, {
  skipPreflight: true,
});

// 3. Serializar transação parcialmente assinada
const serializedTx = unstakeTx.serialize({
  requireAllSignatures: false,
  verifySignatures: false,
}).toString("base64");

// 4. Enviar para relay
await fetch(`${RELAY_URL}/unstake`, {
  method: "POST",
  body: JSON.stringify({
    proof,
    public_inputs,
    unstake,
    partially_signed_tx: serializedTx, // ✅ Adicionar!
  }),
});
```

**Fase 2 - Relay:**
```rust
// 1. Receber partially_signed_tx
let tx_bytes = base64::decode(&request.partially_signed_tx)?;
let mut transaction = Transaction::deserialize(&tx_bytes)?;

// 2. Adicionar assinatura do fee_payer
let recent_blockhash = client.get_latest_blockhash().await?;
transaction.partial_sign(&[fee_payer], recent_blockhash);

// 3. Submeter transação completa (2 assinaturas)
let signature = client.send_and_confirm_transaction(&transaction).await?;
```

#### Arquivos a Modificar
1. `services/web/lib/solana-tx-builder.ts` - Adicionar `buildUnstakeToPoolTransaction`
2. `services/web/app/privacy/page.tsx` - Modificar `handleUnstake` para assinar primeiro
3. `services/relay/src/solana/mod.rs` - Modificar `submit_unstake_to_pool` para aceitar `partially_signed_tx`

---

### Problema 2: **UI não mostra fluxo correto**

#### Descrição
O usuário relatou que a UI mostra "SOL -> Privacy Zone -> USDC" para unstake, quando deveria mostrar:
```
Stake Account -> Privacy Zone -> New Private Note
```

#### Status
✅ **CORRIGIDO!** Adicionei:
- Diagrama visual correto do fluxo de unstake
- Aviso explicando que é necessário assinar como `stake_authority`
- Info box explicando que stake deve estar `deactivated and inactive`

---

### Problema 3: **Transação não aparece no Orb (Solana Explorer)**

#### Descrição
Após o unstake, o usuário não vê a transação no Orb para confirmar on-chain.

#### Causa Raiz
A transação **está falhando** (Problema #1), então nunca chega ao status `"completed"` que dispara:
```typescript
if (status === "completed") {
  const signature = statusJson.data?.signature;
  if (signature) {
    setTransactionSignature(signature); // Isso exibe o modal
  }
}
```

#### Status
⚠️ **Dependente do Problema #1** - Quando a transação passar a funcionar com a assinatura correta, o link do Orb vai aparecer automaticamente no modal de status.

---

## 📋 Próximos Passos (Ordem de Prioridade)

### 1. **Implementar 2-Phase Signing** 🔴 CRÍTICO
- [ ] Criar `buildUnstakeToPoolTransaction` em `solana-tx-builder.ts`
- [ ] Modificar `handleUnstake` para assinar primeiro
- [ ] Modificar `submit_unstake_to_pool` no relay para usar `partially_signed_tx`
- [ ] Testar fluxo completo

### 2. **Testar Unstake End-to-End** 🟠 ALTA
- [ ] ZK proof gerado ✅
- [ ] Usuário assina como stake_authority
- [ ] Relay adiciona assinatura do fee_payer
- [ ] Transação confirmada on-chain
- [ ] Signature exibido no Orb
- [ ] Fundos movidos do stake account para pool PDA
- [ ] Novo commitment no Merkle tree
- [ ] Novo `CloakNote` salvo em localStorage

### 3. **Melhorias de UX** 🟢 BAIXA
- [ ] Loading states melhores durante assinatura
- [ ] Feedback visual quando usuário rejeita assinatura
- [ ] Estimativa de tempo para deactivation (unstake geralmente leva 1-2 epochs)
- [ ] Tutorial/onboarding para primeira vez

---

## 🔍 Informações de Contexto

### Por que Rent Reserve vs Delegated Stake?

Você perguntou sobre stake accounts mostrando 0.0023 SOL vs 0.0993 SOL quando stakeous 0.1 SOL:

**Stake Account Anatomy:**
```
Total Balance = Rent-Exempt Reserve + Delegated Stake

Exemplo após deactivation:
- Total: 0.1 SOL
- Rent Reserve: ~0.00228288 SOL (fixo, não pode ser retirado)
- Delegated Stake: 0.0977 SOL (pode ser unstaked)
```

**Estados:**
1. **Active** - Funds delegados e ganhando rewards
2. **Deactivating** - Esperando end of epoch para deactivate (~2 dias)
3. **Inactive** - Pronto para unstake/withdraw

**Por que mostramos `delegatedStake` na UI?**
- O `rent-exempt reserve` (~0.0023 SOL) **não pode** ser movido para o pool
- Só o `delegatedStake` pode ser unstaked
- Isso evita confusão quando o usuário vê "0.1 SOL staked" mas só consegue unstake 0.0977 SOL

---

## 📖 Referências

- `UNSTAKE_TODO.md` - Detalhes técnicos da implementação
- `ARCHITECTURE_ENCRYPTION.md` - Documentação do ZK circuit
- Solana Stake Program: https://docs.solana.com/developing/runtime-facilities/programs#stake-program
- Similar flow: `handleStake()` em `page.tsx` (Withdraw + Delegate)

