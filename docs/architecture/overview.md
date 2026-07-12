# Architecture Overview

## Design Principles

QSB is built on four foundational architectural patterns:

1. **Layered Architecture**: Separation of concerns across presentation, application, domain, and infrastructure layers.
2. **Hexagonal Architecture (Ports and Adapters)**: Core business logic isolated from external concerns.
3. **Dependency Injection**: Dependencies are injected via constructors and traits, enabling testability.
4. **Trait-based Interfaces**: Every implementation is replaceable through trait abstractions.

## Workspace Structure

```mermaid
graph TD
    subgraph "Layer 0: Foundation"
        BC[blockchain-core]
        CR[cryptography]
        SE[serialization]
        CO[config]
        LO[logging]
    end

    subgraph "Layer 1: Domain"
        WA[wallet]
        TX[transaction]
        CN[consensus]
        ST[storage]
        MP[mempool]
    end

    subgraph "Layer 2: Infrastructure"
        NT[networking]
        MI[miner]
        RP[rpc]
        CL[cli]
        EX[explorer]
        TE[telemetry]
    end

    subgraph "Layer 3: Orchestration"
        NO[node]
        SK[sdk]
    end

    subgraph "Layer 4: Observability & Testing"
        BM[benchmarks]
        TS[testing]
        EG[examples]
        IT[integration-tests]
    end

    NT --> BC & CR & TX
    MI --> BC & CN & CR & MP
    RP --> BC & NO
    CL --> CO & NO
    EX --> BC & ST
    TE --> BC & CN & NT
    NO --> BC & CR & TX & CN & NT & ST & MP & MI & CO & LO & TE
    SK --> BC & TX & CR & WA
    BM --> BC & CN & NT & CR
    TS --> BC
    EG --> NO & CO
    IT --> BC & NO
```

## Dependency Rules

```mermaid
graph LR
    L0[Layer 0<br/>Foundation]
    L1[Layer 1<br/>Domain]
    L2[Layer 2<br/>Infrastructure]
    L3[Layer 3<br/>Orchestration]
    L4[Layer 4<br/>Observability]

    L1 --> L0
    L2 --> L0 & L1
    L3 --> L0 & L1 & L2
    L4 --> L0 & L1 & L2 & L3
```

**Rule**: A crate may only depend on crates in its own layer or lower layers. Never depend on higher layers.

## Crate Responsibilities

| Crate | Responsibility | Layer |
|-------|---------------|-------|
| `blockchain-core` | Core domain models, traits, errors | Foundation |
| `cryptography` | Signature, hash, KEM traits | Foundation |
| `serialization` | Codec abstractions | Foundation |
| `config` | Configuration loading and validation | Foundation |
| `logging` | Structured logging setup | Foundation |
| `wallet` | Key management, signing, verification | Domain |
| `transaction` | Transaction types and validation | Domain |
| `consensus` | Consensus engine trait and algorithms | Domain |
| `storage` | Storage backend traits | Domain |
| `mempool` | Transaction pool management | Domain |
| `networking` | P2P networking layer | Infrastructure |
| `miner` | Block production logic | Infrastructure |
| `rpc` | JSON-RPC server and methods | Infrastructure |
| `cli` | Command-line interface | Infrastructure |
| `explorer` | Block explorer web API | Infrastructure |
| `telemetry` | Metrics collection and export | Infrastructure |
| `node` | Node lifecycle and orchestration | Orchestration |
| `sdk` | Developer client library | Orchestration |
| `benchmarks` | Performance benchmarks | Observability |
| `testing` | Test utilities and fixtures | Observability |
| `examples` | Example applications | Observability |
| `integration-tests` | End-to-end test suite | Observability |

## Crypto Agility

```mermaid
graph TD
    subgraph "Crypto Traits"
        SA[SignatureAlgorithm]
        HF[HashFunction]
        KM[KeyEncapsulationMechanism]
        RG[RandomGenerator]
    end

    subgraph "Implementations"
        ED[Ed25519]
        EC[ECDSA P-256]
        MLD[ML-DSA]
        FAL[Falcon]
        SPH[SPHINCS+]
        SHA[SHA-256]
        SHA3[SHA-3]
        BLK[BLAKE3]
        X25[X25519]
        MLK[ML-KEM]
        HYB[Hybrid KEM]
    end

    SA --> ED & EC & MLD & FAL & SPH
    HF --> SHA & SHA3 & BLK
    KM --> X25 & MLK & HYB
```

All cryptographic operations are defined through traits:

- `SignatureAlgorithm`: `sign()`, `verify()`, `keypair()`
- `HashFunction`: `hash()`, `hash_len()`
- `KemScheme`: `encapsulate()`, `decapsulate()`
- `RandomGenerator`: `fill()`, `try_fill()`

New algorithms are added by implementing these traits. No existing code needs to change.

## Consensus Agility

```mermaid
graph TD
    subgraph "Consensus Trait"
        CE[ConsensusEngine]
    end

    subgraph "Implementations"
        POW[Proof of Work]
        POS[Proof of Stake]
        PBFT[PBFT]
        RAFT[Raft]
        HOT[HotStuff]
        RES[Research Algorithms]
    end

    CE --> POW & POS & PBFT & RAFT & HOT & RES
```

Consensus algorithms are isolated crates implementing the `ConsensusEngine` trait. Switching consensus is a configuration change.

## Storage Agility

```mermaid
graph TD
    subgraph "Storage Traits"
        BS[BlockStore]
        TS[TransactionStore]
        SS[StateStore]
        KV[KeyValueStore]
        SN[SnapshotStore]
    end

    subgraph "Implementations"
        ROC[RocksDB]
        SLE[Sled]
        MEM[In-Memory]
        FUT[Future Backends]
    end

    BS & TS & SS & KV & SN --> ROC & SLE & MEM & FUT
```

Storage backends implement the `StorageBackend` trait. Multiple backends can coexist.

## Networking

```mermaid
graph TD
    subgraph "Network Traits"
        TR[Transport]
        PD[PeerDiscovery]
        MC[MessageCodec]
        NS[NetworkService]
        PM[PeerManager]
        SY[SyncManager]
    end

    subgraph "libp2p Protocols"
        ID[Identify]
        PN[Ping]
        MD[mDNS]
        GS[Gossipsub]
        RR[Request-Response]
        AN[AutoNAT]
        RL[Relay]
    end

    NS --> TR & PD & MC & PM & SY
    PD --> MD & ID
    PM --> PN & AN & RL
    SY --> GS & RR
```

Networking uses libp2p with modular protocols. Each protocol is optional and swappable.

## Plugin System

```mermaid
graph TD
    subgraph "Plugin Registry"
        PR[PluginRegistry]
    end

    subgraph "Plugin Types"
        SIG[Signature Plugins]
        HAS[Hash Plugins]
        KEM[KEM Plugins]
        CON[Consensus Plugins]
        STO[Storage Plugins]
        NET[Network Plugins]
    end

    PR --> SIG & HAS & KEM & CON & STO & NET
```

Plugins are discovered at runtime via configuration. No recompilation required.

## Future Expansion Strategy

1. **Crypto**: Add new NIST algorithms by implementing existing traits
2. **Consensus**: Add new algorithms as independent crates
3. **Storage**: Add new backends by implementing `StorageBackend`
4. **Networking**: Add new protocols by implementing `NetworkService`
5. **Research**: Add simulators as optional crates in `examples/`

## Error Handling

```mermaid
graph TD
    subgraph "Error Hierarchy"
        CE[CoreError]
        CR[CryptoError]
        WE[WalletError]
        TXE[TransactionError]
        CNE[ConsensusError]
        NE[NetworkError]
        SE[StorageError]
        RE[RpcError]
        COE[ConfigError]
        LE[LoggingError]
        TE[TelemetryError]
    end
```

- Libraries use `thiserror` for typed errors
- Applications use `anyhow` for context-rich errors
- All errors are `Send + Sync + 'static`
- No panics in library code

## Security

- `zeroize` for sensitive data
- Constant-time comparisons with `subtle`
- Input validation at all boundaries
- No secrets in logs
- All unsafe blocks documented
- No hardcoded credentials
