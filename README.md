# ₿ PaperMarket

> Marketplace descentralizado powered by **Bitcoin Paper Cash (BPC)** — uma blockchain simulada com mineração, carteiras, contratos e sistema de escrow.

```
Estética: Amarelo e Preto · Anarcocapitalista · Free Market · No Rulers
```

---

## 📦 Stack

| Camada       | Tecnologia                        |
|--------------|-----------------------------------|
| Frontend     | Next.js 14 (App Router)           |
| Backend      | Rust + Actix-web                  |
| Blockchain   | Rust (engine própria, SHA-256 PoW)|
| Banco        | SQLite via SQLx                   |
| Comunicação  | REST API + WebSocket (tempo real) |
| Criptografia | secp256k1 + BIP-39 simulado       |

---

## 🗂️ Estrutura do Monorepo

```
papermarket/
│
├── README.md
├── .gitignore
├── .env.example
│
├── apps/
│   ├── web/                        # Frontend Next.js
│   │   ├── public/
│   │   ├── src/
│   │   │   ├── app/                # App Router (Next.js 14)
│   │   │   │   ├── layout.tsx
│   │   │   │   ├── page.tsx                # Marketplace (home)
│   │   │   │   ├── wallet/
│   │   │   │   │   └── page.tsx            # Carteira BPC
│   │   │   │   ├── mining/
│   │   │   │   │   └── page.tsx            # Aba de mineração
│   │   │   │   ├── explorer/
│   │   │   │   │   └── page.tsx            # Block Explorer
│   │   │   │   ├── contracts/
│   │   │   │   │   └── page.tsx            # Smart Contracts / Escrow
│   │   │   │   └── auth/
│   │   │   │       ├── register/
│   │   │   │       │   └── page.tsx        # Criar conta
│   │   │   │       └── login/
│   │   │   │           └── page.tsx        # Entrar
│   │   │   │
│   │   │   ├── components/
│   │   │   │   ├── layout/
│   │   │   │   │   ├── Navbar.tsx
│   │   │   │   │   ├── Sidebar.tsx
│   │   │   │   │   └── Footer.tsx
│   │   │   │   ├── marketplace/
│   │   │   │   │   ├── ProductCard.tsx
│   │   │   │   │   ├── ProductGrid.tsx
│   │   │   │   │   ├── SearchBar.tsx
│   │   │   │   │   ├── Filters.tsx
│   │   │   │   │   └── ProductDetail.tsx
│   │   │   │   ├── wallet/
│   │   │   │   │   ├── BalanceCard.tsx
│   │   │   │   │   ├── SendForm.tsx
│   │   │   │   │   └── TxHistory.tsx
│   │   │   │   ├── mining/
│   │   │   │   │   ├── MiningPanel.tsx
│   │   │   │   │   ├── MiningStats.tsx
│   │   │   │   │   └── Mempool.tsx
│   │   │   │   ├── explorer/
│   │   │   │   │   ├── BlockList.tsx
│   │   │   │   │   └── TxDetail.tsx
│   │   │   │   ├── contracts/
│   │   │   │   │   ├── EscrowCard.tsx
│   │   │   │   │   └── ContractForm.tsx
│   │   │   │   └── ui/
│   │   │   │       ├── Button.tsx
│   │   │   │       ├── Input.tsx
│   │   │   │       ├── Tag.tsx
│   │   │   │       ├── Modal.tsx
│   │   │   │       └── Toast.tsx
│   │   │   │
│   │   │   ├── hooks/
│   │   │   │   ├── useWallet.ts
│   │   │   │   ├── useMining.ts
│   │   │   │   └── useWebSocket.ts
│   │   │   │
│   │   │   ├── lib/
│   │   │   │   ├── api.ts              # Client HTTP para o backend
│   │   │   │   ├── bip39.ts            # Geração das 12 palavras (client-side)
│   │   │   │   ├── crypto.ts           # Derivação de chaves (client-side)
│   │   │   │   └── format.ts           # Formatação de BPC, endereços, etc.
│   │   │   │
│   │   │   ├── store/
│   │   │   │   └── walletStore.ts      # Zustand: estado global da carteira
│   │   │   │
│   │   │   └── styles/
│   │   │       └── globals.css         # Tema amarelo/preto, variáveis CSS
│   │   │
│   │   ├── next.config.ts
│   │   ├── tailwind.config.ts
│   │   ├── tsconfig.json
│   │   └── package.json
│   │
│   └── api/                            # Backend Rust + Actix-web
│       ├── src/
│       │   ├── main.rs                 # Entry point, configuração do servidor
│       │   │
│       │   ├── routes/
│       │   │   ├── mod.rs
│       │   │   ├── auth.rs             # POST /auth/register, /auth/login
│       │   │   ├── wallet.rs           # GET /wallet/:address, POST /wallet/send
│       │   │   ├── marketplace.rs      # CRUD de produtos e ordens
│       │   │   ├── chain.rs            # GET /chain/blocks, /chain/tx/:hash
│       │   │   ├── mining.rs           # POST /mining/submit, GET /mining/job
│       │   │   └── contracts.rs        # POST /contracts/escrow, GET /contracts/:id
│       │   │
│       │   ├── models/
│       │   │   ├── mod.rs
│       │   │   ├── user.rs             # User, Wallet, Keypair
│       │   │   ├── product.rs          # Product, Order, Review
│       │   │   ├── transaction.rs      # Tx, UTXO, TxInput, TxOutput
│       │   │   ├── block.rs            # Block, BlockHeader
│       │   │   └── contract.rs         # EscrowContract, ContractState
│       │   │
│       │   ├── blockchain/
│       │   │   ├── mod.rs
│       │   │   ├── chain.rs            # Lógica principal da chain
│       │   │   ├── pow.rs              # Proof of Work (SHA-256)
│       │   │   ├── mempool.rs          # Fila de transações pendentes
│       │   │   ├── utxo.rs             # Gerenciamento de UTXOs
│       │   │   └── contracts.rs        # Engine de smart contracts
│       │   │
│       │   ├── crypto/
│       │   │   ├── mod.rs
│       │   │   ├── keys.rs             # Geração de par de chaves secp256k1
│       │   │   ├── bip39.rs            # Wordlist + derivação de seed
│       │   │   └── signing.rs          # Assinatura e verificação de TXs
│       │   │
│       │   ├── db/
│       │   │   ├── mod.rs
│       │   │   ├── connection.rs       # Pool SQLite via SQLx
│       │   │   └── migrations/
│       │   │       ├── 001_users.sql
│       │   │       ├── 002_products.sql
│       │   │       ├── 003_transactions.sql
│       │   │       ├── 004_blocks.sql
│       │   │       └── 005_contracts.sql
│       │   │
│       │   ├── ws/
│       │   │   └── handler.rs          # WebSocket: mempool, blocos em tempo real
│       │   │
│       │   └── errors.rs               # Tipos de erro centralizados
│       │
│       ├── Cargo.toml
│       └── .env.example
│
├── packages/                           # Código compartilhado (futuro)
│   └── bpc-types/                      # Tipos compartilhados (se usar codegen)
│
└── docs/
    ├── architecture.md                 # Diagrama da arquitetura
    ├── bpc-protocol.md                 # Especificação do protocolo BPC
    ├── escrow-contract.md              # Spec do CONTRACT_ESCROW_v1
    └── api-reference.md                # Referência completa da API REST
```

---

## 🔑 Sistema de Contas

Criação de conta é **100% client-side**. Nenhuma chave privada toca o servidor.

```
1. Usuário escolhe um username
2. Usuário define uma masterkey (senha forte)
3. Sistema gera 12 palavras aleatórias (BIP-39 simulado) — exibidas UMA VEZ
4. Da seed das 12 palavras → derivação do par de chaves (secp256k1)
5. Endereço BPC gerado a partir da chave pública
6. Servidor armazena: username + endereço público (NUNCA a chave privada)
```

---

## ₿ Bitcoin Paper Cash (BPC)

Token simulado com comportamento idêntico ao Bitcoin real:

| Característica     | Valor                              |
|--------------------|------------------------------------|
| Modelo             | UTXO (como Bitcoin)                |
| Consenso           | Proof of Work (SHA-256)            |
| Recompensa inicial | 6.25 BPC por bloco                 |
| Halving            | A cada 210.000 blocos              |
| Dificuldade        | Ajuste a cada 2.016 blocos         |
| Criptografia       | secp256k1                          |
| Endereços          | Prefixo `1BPC...`                  |
| Seed phrase        | 12 palavras (BIP-39 simulado)      |

---

## 📜 Contratos — CONTRACT_ESCROW_v1

Contrato de escrow multisig 2/3 registrado on-chain:

```json
{
  "contract_id": "uuid-v4",
  "version": "CONTRACT_ESCROW_v1",
  "parties": {
    "seller_pubkey": "04abc...",
    "buyer_pubkey": "04def...",
    "arbiter_pubkey": "PAPERMARKET_ARB_MASTER_01"
  },
  "terms": {
    "amount_bpc": 0.0042,
    "item_hash": "sha256-do-descritivo-do-produto",
    "item_description": "Teclado Mecânico TKL - Novo",
    "created_at_block": 1848,
    "expires_at_block": 1948,
    "multisig_threshold": "2_OF_3"
  },
  "state": "PENDING | LOCKED | RELEASED | DISPUTED | REFUNDED",
  "signatures": [],
  "tx_hash": "hash-da-tx-que-criou-o-contrato"
}
```

**Fluxo do Escrow:**
```
Comprador envia BPC → Contrato LOCKED
    → Vendedor entrega produto
        → Comprador confirma → RELEASED → BPC vai pro vendedor
        → Disputa → Árbitro decide → RELEASED ou REFUNDED
    → Prazo expira sem confirmação → REFUNDED automaticamente
```

---

## 🛣️ API REST — Rotas Principais

```
AUTH
  POST   /api/auth/register          Criar conta
  POST   /api/auth/login             Autenticar

WALLET
  GET    /api/wallet/:address        Saldo e UTXOs
  GET    /api/wallet/:address/txs    Histórico de transações
  POST   /api/wallet/send            Criar e transmitir TX

MARKETPLACE
  GET    /api/products               Listar produtos (filtros, paginação)
  GET    /api/products/:id           Detalhe do produto
  POST   /api/products               Criar listagem
  PUT    /api/products/:id           Editar listagem
  DELETE /api/products/:id           Remover listagem
  POST   /api/orders                 Criar ordem de compra

BLOCKCHAIN
  GET    /api/chain/info             Info geral da chain
  GET    /api/chain/blocks           Listar blocos
  GET    /api/chain/blocks/:height   Detalhe do bloco
  GET    /api/chain/tx/:hash         Detalhe de transação

MINERAÇÃO
  GET    /api/mining/job             Pegar trabalho atual (header + target)
  POST   /api/mining/submit          Submeter bloco minerado

CONTRATOS
  POST   /api/contracts/escrow       Criar contrato de escrow
  GET    /api/contracts/:id          Consultar contrato
  POST   /api/contracts/:id/sign     Assinar contrato (comprador/vendedor/árbitro)
  POST   /api/contracts/:id/dispute  Abrir disputa

WEBSOCKET
  WS     /ws                         Eventos: novos blocos, TXs, mempool
```

---

## 🚀 Como rodar (desenvolvimento local)

### Pré-requisitos
- Rust 1.75+
- Node.js 20+
- pnpm

### Backend
```bash
cd apps/api
cp .env.example .env
cargo run
# Servidor em http://localhost:8080
```

### Frontend
```bash
cd apps/web
pnpm install
pnpm dev
# App em http://localhost:3000
```

---

## 📋 Roadmap

- [x] Definição da arquitetura
- [ ] Estrutura do repositório
- [ ] Migrations SQLite
- [ ] Engine blockchain (blocos + PoW)
- [ ] Sistema de UTXOs
- [ ] Criptografia (BIP-39 + secp256k1)
- [ ] Auth (registro + login)
- [ ] API REST completa
- [ ] Frontend base (tema + layout)
- [ ] Marketplace (listagem + compra)
- [ ] Sistema de mineração (frontend)
- [ ] Block Explorer
- [ ] Smart Contracts (Escrow 2/3)
- [ ] WebSocket (tempo real)
- [ ] Sistema de reputação on-chain

---

## ⚖️ Filosofia

> *"Um mercado livre de verdade não pede permissão."*

PaperMarket é um projeto educacional para demonstrar como Bitcoin, criptografia de chave pública, UTXO e contratos multisig funcionam na prática — sem depender de nenhuma blockchain real.

---

**Licença:** MIT