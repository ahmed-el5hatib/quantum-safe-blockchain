# cryptography

Cryptographic primitives, traits, and abstractions.

## Architecture

```mermaid
classDiagram
    class SignatureAlgorithm {
        <<trait>>
        +generate_keypair() KeyPair
        +sign(private_key, message) Signature
        +verify(public_key, message, signature) bool
        +algorithm_name() str
    }
    class HashFunction {
        <<trait>>
        +hash(data) HashDigest
        +hash_len() usize
        +algorithm_name() str
    }
    class KeyEncapsulationMechanism {
        <<trait>>
        +generate_keypair() KeyPair
        +encapsulate(public_key) EncapsulatedKey
        +decapsulate(private_key, ciphertext) Vec
        +algorithm_name() str
    }
    class PublicKey {
        <<trait>>
        +as_bytes() [u8]
        +algorithm_name() str
    }
    class PrivateKey {
        <<trait>>
        +as_bytes() [u8]
        +algorithm_name() str
    }
    class Signature {
        <<trait>>
        +as_bytes() [u8]
        +algorithm_name() str
    }
    class KeyPair {
        +public_key PublicKey
        +private_key PrivateKey
    }
    SignatureAlgorithm --> PublicKey
    SignatureAlgorithm --> PrivateKey
    SignatureAlgorithm --> Signature
    KeyEncapsulationMechanism --> PublicKey
    KeyEncapsulationMechanism --> PrivateKey
    SignatureAlgorithm --> KeyPair
    KeyEncapsulationMechanism --> KeyPair
```

## Future Roadmap

- Add Ed25519 implementation
- Add ECDSA P-256 implementation
- Add SHA-256 implementation
- Add BLAKE3 implementation
- Add ML-DSA implementation
- Add ML-KEM implementation