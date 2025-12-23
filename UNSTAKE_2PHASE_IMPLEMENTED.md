# 🎉 Unstake 2-Phase Signing - IMPLEMENTADO!

## ✅ O Que Foi Feito

Implementei o fluxo completo de 2-phase signing para o unstake, resolvendo o problema crítico de falta de assinatura do `stake_authority`.

### 1. **Frontend** (`services/web`)

#### Novo arquivo: `lib/unstake-tx-builder.ts`
- ✅ Função `buildUnstakeToPoolTransaction()` 
- ✅ Constrói a transação UnstakeToPool com:
  - Proof (260 bytes)
  - Public inputs (104 bytes)
  - Stake account
  - Compute budget instructions
  - Todos os accounts necessários
- ✅ **Usa a public key do USUÁRIO como feePayer** (relay atualiza depois)

#### Modificado: `app/privacy/page.tsx`
- ✅ Adicionado `signTransaction` ao destructuring do `useWallet()`
- ✅ Modificado `handleUnstake()` para:
  1. Gerar ZK proof ✅
  2. Construir public inputs (104 bytes) ✅
  3. **Criar transação UnstakeToPool** ✅
  4. **Solicitar assinatura do usuário** (stake_authority) ✅
  5. Serializar transação parcialmente assinada ✅
  6. Enviar `partially_signed_tx` ao relay ✅

- ✅ Melhorias na UI:
  - Diagrama visual correto do fluxo
  - Avisos sobre necessidade de assinatura
  - Status "signing" durante assinatura do usuário
  - Tratamento de erros quando usuário rejeita

### 2. **Relay** (`services/relay`)

#### Modificado: `src/api/unstake.rs`
- ✅ Campo `partially_signed_tx: Option<String>` na struct `UnstakeRequest`
- ✅ Metadata do job inclui `partially_signed_tx` quando fornecido
- ✅ Logs informativos

#### Modificado: `src/solana/mod.rs`
- ✅ Função `submit_unstake_to_pool()` agora:
  1. Verifica se há `partially_signed_tx` no job
  2. Se sim:
     - Desserializa a transação (base64 → bytes → Transaction)
     - **Atualiza `feePayer` para o relay** (frontend usa user's key como placeholder)
     - Atualiza o blockhash
     - **Adiciona assinatura do fee_payer** (relay)
     - Verifica que tem 2 assinaturas
     - Submete transação completa ✅
  3. Se não:
     - Retorna erro informativo (exige 2-phase signing)

- ✅ Imports corretos: `base64`, `bincode`

---

## 🎯 **IMPORTANTE: Sem Envs Adicionais Necessárias!**

**Não é necessário configurar nenhuma env adicional!** ✅

A implementação usa a public key do usuário como `feePayer` temporário ao construir a transação no frontend. Quando o relay recebe a transação parcialmente assinada, ele:

1. Atualiza o `feePayer` para sua própria chave
2. Adiciona sua assinatura
3. Submete a transação

Isso mantém consistência com os outros fluxos (stake, swap, transfer) que não exigem configuração extra.

---

## 🔄 Fluxo Completo (Ponta a Ponta)

### Fase 1 - Frontend (Usuário)
```
1. Usuário seleciona stake account inativo
2. Clica em "Unstake to Pool Privately"
3. ZK proof é gerado no TEE ✅
4. Transação UnstakeToPool é construída
5. 🔐 WALLET PROMPT: "Sign transaction to authorize withdrawal"
6. Usuário assina como stake_authority
7. Transação parcialmente assinada é serializada (base64)
8. Enviada ao relay via /unstake endpoint
```

### Fase 2 - Relay (Automático)
```
1. Relay recebe proof + partially_signed_tx
2. Cria job no banco de dados
3. Worker processa o job:
   - Desserializa transação
   - Adiciona assinatura do fee_payer (relay)
   - Submete à blockchain ✅
4. Fundos movidos: Stake Account → Shield Pool
5. Nova commitment adicionada à Merkle tree
6. Frontend salva novo CloakNote
```

---

## 🧪 Como Testar

### 1. **Compilar**
```bash
# Relay
cd services/relay && cargo build --release

# Frontend
cd services/web && npm run build
```

### 2. **Preparar Stake Account**
```bash
# 1. Criar stake account via UI normal de Stake
# 2. Esperar ativação (~1 epoch)
# 3. Deactivar stake:
solana deactivate-stake <STAKE_ACCOUNT_PUBKEY>

# 4. Esperar deactivation (~1-2 epochs)
# 5. Verificar status:
solana stake-account <STAKE_ACCOUNT_PUBKEY>
# Deve mostrar "inactive"
```

### 3. **Testar Unstake**
```
1. Abrir /privacy
2. Clicar em "Unstake" tab
3. Verificar que stake account aparece na lista
4. Selecionar o account
5. Clicar "Unstake to Pool Privately"
6. ⚠️  IMPORTANTE: ASSINAR quando wallet pedir
7. Aguardar processamento (~30s para proof)
8. ✅ Verificar:
   - Transaction signature exibido
   - Link para Solscan/explorer
   - Novo note em localStorage
   - Balance atualizado
```

### 4. **Verificar On-Chain**
```bash
# Verificar transação no explorer
# Deve conter:
# - 2 signatures (user + relay)
# - Instruction: UnstakeToPool (discriminant 10)
# - Stake account balance reduzido
# - Pool PDA balance aumentado
```

---

## ✅ Checklist de Funcionalidades

### ZK Circuit
- [x] Aceita `unstake_params`
- [x] Skips Merkle/nullifier verification
- [x] Verifica commitment correto

### Frontend
- [x] UI com diagrama de fluxo correto
- [x] Seletor de stake accounts (só inactive)
- [x] Gera ZK proof
- [x] Constrói transação UnstakeToPool
- [x] **Solicita assinatura do usuário** 🔐
- [x] Serializa e envia ao relay
- [x] Salva novo CloakNote

### Relay
- [x] Endpoint `/unstake` aceita `partially_signed_tx`
- [x] Armazena no job metadata
- [x] **Desserializa e adiciona assinatura** 🔐
- [x] Verifica 2 assinaturas
- [x] Submete à blockchain

### UX/UI
- [x] Diagrama visual: Stake Account → Privacy Zone → New Note
- [x] Aviso sobre necessidade de assinatura
- [x] Status "signing" durante assinatura
- [x] Exibe transaction signature ao finalizar
- [x] Link para Orb/Solscan
- [x] Tratamento de rejeição de assinatura

---

## 🐛 Possíveis Problemas e Soluções

### 1. "Transaction signature verification failure"
**Causa:** Falta uma das assinaturas

**Solução:**
- Verificar que `partially_signed_tx` foi enviado
- Verificar logs do relay: "Transaction has both signatures"
- Se usuário rejeitou, aparecerá erro específico

### 2. "Wallet does not support transaction signing"
**Causa:** Wallet não suporta `signTransaction`

**Solução:**
- Use Phantom, Solflare, ou outra wallet moderna
- Algumas wallets antigas não suportam signing sem send

### 3. "Stake account not inactive"
**Causa:** Stake ainda não completou deactivation

**Solução:**
- Esperar mais tempo (pode demorar 2-3 epochs = ~4-6 dias)
- Verificar: `solana stake-account <PUBKEY>`
- Só aparecerá na UI quando status for "inactive"

### 4. Transação falha com "insufficient funds"
**Causa:** Rent-exempt reserve não pode ser retirado

**Solução:**
- Frontend já calcula corretamente (`delegatedStake`, não `balance`)
- Se ainda falhar, verificar que stake foi delegated antes

---

## 📊 Logs Esperados

### Frontend Console
```
[Unstake] Total amount: 97022526
[Unstake] Fee (0.5%): 485112
[Unstake] Deposit amount: 96537414
[Unstake] Generated commitment: <hash>
[Unstake] ✅ ZK proof generated successfully
[Unstake] 📝 Building UnstakeToPool transaction...
[Unstake] ✍️  Requesting user signature as stake_authority...
[Unstake] ✅ User signed transaction
[Unstake] 📦 Serialized partially signed transaction
[Unstake] Request ID: <uuid>
[Unstake] Poll attempt 1: status=processing
...
[Unstake] Poll attempt N: status=completed
✅ Unstake completed! 0.0965 SOL now in the shield pool
```

### Relay Logs
```
INFO relay::api::unstake: Received unstake request
INFO relay::api::unstake: ✅ Partially signed transaction included in job metadata
INFO relay::api::unstake: Unstake request queued successfully
...
INFO relay::solana: Starting unstake-to-pool flow for job <uuid>
INFO relay::solana: ✅ Received partially signed transaction from frontend
INFO relay::solana: Adding relay fee_payer signature
INFO relay::solana: ✅ Transaction has both signatures (user + relay)
INFO relay::solana: ✅ Unstake-to-pool transaction confirmed: <signature>
```

---

## 🎯 Próximos Passos (Opcional)

### Melhorias de UX
- [ ] Mostrar preview da transação antes de assinar
- [ ] Estimativa de gas fees
- [ ] Progresso visual durante assinatura
- [ ] Tutorial/onboarding para primeira vez

### Otimizações
- [ ] Batch múltiplos unstakes em uma transação
- [ ] Permitir partial unstake (não todo o stake)
- [ ] Auto-refresh da lista de stake accounts

### Segurança
- [ ] Rate limiting no endpoint /unstake
- [ ] Verificação adicional de ownership do stake account
- [ ] Alertas se stake account não pertence ao user

---

## 📚 Referências

- `UNSTAKE_TODO.md` - Detalhes técnicos originais
- `UNSTAKE_STATUS.md` - Status e contexto
- Solana Stake Program: https://docs.solana.com/developing/runtime-facilities/programs#stake-program
- SP1 TEE docs: https://docs.succinct.xyz/

---

**Status:** ✅ IMPLEMENTADO  
**Testado:** ⚠️ Aguardando teste do usuário  
**Prioridade:** 🔴 CRÍTICO - Resolve bloqueio de unstake

