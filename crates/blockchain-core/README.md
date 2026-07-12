# blockchain-core

Core domain models, traits, errors, and events for QSB.

## Architecture

```mermaid
classDiagram
    class BlockT {
        <<trait>>
        +header() BlockHeader
        +transactions() [Transaction]
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
        +version u32
        +previous_hash HashDigest
        +merkle_root MerkleRoot
        +timestamp u64
        +bits u32
        +nonce u64
    }
    class Transaction {
        +version u32
        +inputs [TransactionInput]
        +outputs [TransactionOutput]
        +lock_time u64
    }
    class MerkleRoot
    class HashDigest
    class EventBus {
        <<trait>>
        +publish(Event) CoreResult
        +subscribe(EventHandler) CoreResult
    }
    class Event {
        <<enum>>
        BlockMined
        TransactionReceived
        PeerConnected
        PeerDisconnected
        ChainReorg
        ConsensusFinalized
    }
    BlockT --> BlockHeader
    BlockT --> Transaction
    TransactionT --> TransactionInput
    TransactionT --> TransactionOutput
    EventBus --> Event
```

## Future Roadmap

- Add generic block validation hooks
- Add state transition traits
- Add checkpoint traits
- Add fork resolution traits