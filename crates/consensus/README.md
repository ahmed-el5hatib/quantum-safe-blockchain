# consensus

Consensus algorithms and traits.

## Architecture

```mermaid
classDiagram
    class ConsensusEngine {
        <<trait>>
        +name() str
        +validate_block(block) CoreResult
        +propose_block(parent) CoreResult Block
        +finalize_block(block) CoreResult
        +should_finalize(block) CoreResult bool
    }
    class ConsensusParams {
        <<trait>>
        +validate() CoreResult
    }
    class ConsensusState {
        <<trait>>
        +apply_block(block) CoreResult
        +revert_block(block) CoreResult
    }
```

## Future Roadmap

- Proof of Work implementation
- Proof of Stake implementation
- PBFT implementation
- Raft implementation
- HotStuff implementation