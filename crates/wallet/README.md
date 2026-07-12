# wallet

Wallet functionality: key generation, signing, and verification.

## Architecture

```mermaid
classDiagram
    class Wallet {
        <<trait>>
        +create_account(name) CoreResult AccountId
        +get_account(id) CoreResult Option Account
        +list_accounts() CoreResult [AccountId]
        +remove_account(id) CoreResult
    }
    class Account {
        <<trait>>
        +id() AccountId
        +name() str
        +address() Address
        +public_key() PublicKey
    }
    class Address {
        <<trait>>
        +as_bytes() [u8]
        +to_string() String
        +from_string(s) CoreResult Address
    }
    class Signer {
        <<trait>>
        +sign(private_key, message) CoreResult Signature
    }
    class Verifier {
        <<trait>>
        +verify(public_key, message, signature) CoreResult
    }
    class WalletManager {
        <<trait>>
        +create_wallet(password) CoreResult Box Wallet
        +open_wallet(data, password) CoreResult Box Wallet
    }
```

## Future Roadmap

- Ed25519 key generation
- ECDSA key generation
- Address derivation
- Multi-signature support
- Hardware wallet integration