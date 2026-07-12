# sdk

Developer SDK for interacting with QSB nodes.

## Architecture

```mermaid
classDiagram
    class Client {
        <<trait>>
        +connect(addr) CoreResult
        +disconnect() CoreResult
        +is_connected() bool
        +send_transaction(tx) CoreResult HashDigest
        +get_block(hash) CoreResult Option [u8]
        +get_chain_state() CoreResult ChainState
        +list_peers() CoreResult [PeerId]
    }
```

## Future Roadmap

- HTTP client implementation
- WebSocket client implementation
- Wallet SDK helpers
- Type-safe request builders