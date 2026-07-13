# blockchain-core

Core domain models, traits, errors, and events for QSB.

## Architecture

```mermaid
classDiagram
    class BlockT {
        <<trait>>
        +header() BlockHeader
        +transactions() [Transaction]
        +metadata() BlockMetadata
        +block_hash() BlockHash
        +hash() HashDigest
        +validate() CoreResult
    }
    class TransactionT {
        <<trait>>
        +hash() HashDigest
        +inputs() [TransactionInput]
        +outputs() [TransactionOutput]
        +validate() CoreResult
    }
    class BlockHeader {
        +version Version
        +height Height
        +timestamp Timestamp
        +previous_hash PreviousHash
        +merkle_root MerkleRoot
        +difficulty Difficulty
        +nonce Nonce
    }
    class Block {
        +header BlockHeader
        +transactions [Transaction]
        +metadata BlockMetadata
        +block_hash BlockHash
    }
    class BlockMetadata
    class ChainMetadata
    class GenesisBlock
    class GenesisConfig
    class Transaction
    class MerkleRoot
    class HashDigest
    class EventBus {
        <<trait>>
        +publish(Event) CoreResult
        +subscribe(EventHandler) CoreResult
    }
    class Event
    BlockT --> BlockHeader
    BlockT --> Block
    Block --> BlockMetadata
    Block --> BlockHash
    TransactionT --> Transaction
    EventBus --> Event
```

## Block Anatomy

A QSB block consists of:

1. **Header**: Immutable metadata describing the block
   - `version`: Protocol version
   - `height`: Block height in the chain
   - `timestamp`: Unix timestamp
   - `previous_hash`: Hash of the previous block
   - `merkle_root`: Root of the transaction Merkle tree
   - `difficulty`: Mining difficulty target
   - `nonce`: Proof-of-work nonce

2. **Transactions**: List of transactions included in the block

3. **Metadata**: Block metadata
   - `block_size`: Serialized size in bytes
   - `transaction_count`: Number of transactions

4. **BlockHash**: Cryptographic hash of the block header

## Genesis

The genesis block is the first block in the chain. It is generated deterministically from a `GenesisConfig`, ensuring all nodes agree on the initial state without a trusted setup.

## Crypto Agility Integration

Block hashing uses the [`HashFunction`] trait from the cryptography crate. This means the hash algorithm can be changed without modifying any blockchain logic.

## Future Roadmap

- Add generic block validation hooks
- Add state transition traits
- Add checkpoint traits
- Add fork resolution traits
- Add Merkle tree implementation
