# FilterThis

Code is super WIP. Don't even try to run it.

## Overview

The following is a mechanism for committing to large data objects (e.g., images) using Bitcoin Taproot outputs without embedding the data directly on-chain. The commitment is constructed using the Taproot script tree, where each Tapleaf contains a chunk of the data. A non-interactive, probabilistically sound attestation allows third parties to verify with high confidence that a given object was indeed committed to in the Taproot tree.

This enables decentralized publication and verification of large assets (e.g., inscriptions) using already available taproot tools, with minimal on-chain footprint and no reliance on consensus enforcement. Additionally, the commitment is privacy preserving. Only the owner and the buyer know what the data is committed to.

## Motivation

Traditional data inscriptions on Bitcoin (e.g., Ordinals) require embedding the full asset in the witness or in `OP_RETURN` outputs, increasing on-chain storage costs and reducing buyer/seller privacy. This approach shifts data verification off-chain, while retaining on-chain cryptographic commitment via Taproot.


## Construction

### Data Chunking

Given a data object `D` (e.g., a WebP image), we split it into fixed-size chunks:

```
D = D₀ || D₁ || ... || D_{n-1},    where each Dᵢ ∈ {0,1}^L
```

Let `L` be the chunk size (e.g., 64 bytes). The total number of chunks is `n = ceil(|D| / L)`.

Each chunk `Dᵢ` will be encoded into a Tapleaf script:

```
scriptᵢ = OP_PUSHBYTES_{|Dᵢ|} Dᵢ
```

These scripts are compiled into Tapleaves and inserted into the Taproot script tree.
Note: we have to be careful here that no chunk is "truthy" otherwise if someone learns about this leaf in the verification process they can just spend the output.

### Taproot Tree Construction

* Let the internal key be derived from the owner / minter 's keypair.
* The Tapleaves are the data chunks.

The Taproot output `P` is created:

```
P = Taproot_Tweak(internal_key, Merkle_root({script_0..n}))
```

### Non-Interactive Attestation (Fiat-Shamir Transcript)

To provide off-chain assurance that `D` is committed in the tree, the committer (minter) generates a **Fiat-Shamir transcript** of Merkle openings.

#### a. Initialization

* Let `H` be a cryptographic hash function (e.g., SHA-256).
* Let `s = H("TaprootDataProof" || H(D))` be the seed of the challenge protocol. 
// I think you need something else here that is publicly verifiable. Otherwise the prover knows what k chunks are being queried and can just commit to those(?). 
// Maybe it should be `s = H("TaprootDataProof" || H(D) || bitcoin_block_hash)`

#### b. Non-Interactive Challenge-Response

For `k` rounds:

```
For i in 0 .. k-1:
    queryᵢ = H(s || response₀ || ... || response_{i-1}) mod n
    leaf_index = queryᵢ
    responseᵢ = {
        index: leaf_index,
        chunk: D_{leaf_index},
        proof: MerkleProof(leaf_index)
    }
```

Each `responseᵢ` includes:

* `index`: the Tapleaf index (0-based)
* `chunk`: the raw bytes of `Dᵢ`
* `proof`: the Merkle proof from the leaf to the Taproot root

The transcript consists of all `(queryᵢ, responseᵢ)` pairs.

#### c. Binding to the Data

Because the challenges are deterministically derived from `H(D)`, the transcript is **bound to the exact data D**, including its encoding and chunking.

## Verification

To verify that `D` was committed in `P`, a verifier:

1. Computes `s`
2. Replays the Fiat-Shamir transcript using the hash chain
3. For each query:

   * Checks that the `chunk` matches `D_{queryᵢ}`
   * Verifies that the Merkle proof links the chunk’s Tapleaf to the Taproot root of `P`

If all `k` responses verify, the verifier accepts with high confidence that `D` was fully committed to in `P`.
Note: perhaps the transcript should be provided in an opreturn when spent? But that clearly is fingerprintable and some mining pools may not include in blocks due to philosophical issues. But the data is no longer on-chain so maybe its fine. Who knows. 

In practice, a minter will provide their output + transcript to a market for ordinals/NFTs. Buyers will verify the data is being commited to. Using Ordinal theory buyers can verify their sats are indeed linked to the orignial inscriptionless-incription.


## Security Model

The soundness of this scheme depends on:

* The collision resistance of `H`
* The minter's inability to predict challenges before committing to `P`
* The number of sampled leaves `k`

If a malicious committer includes invalid or incomplete data in some leaves, but correctly responds to `k` Fiat-Shamir challenges, the probability they escape detection is:

```math
Pr["Fake Passes"] = (m / n)^k
```

Where `m` is the number of valid leaves, and `n` is total leaves.

This is negligible if `k` is large and `m << n`.

### Example

Chunk size = 64 bytes.

* D = 2^15 = 32768 bytes
* n = 512 chunks
* k = 32 challenges
* m = 50 **valid** chunks. i.e. the minter only put valid data in 50 chunks the rest are bogus or non-existent.

The probability of getting away with it is `(50/512)^32 ≈ 2.3 × 10^-47`, which is essentially zero.

## Advantages

* **Chain-efficient**: Its just a Taproot commitment.
* **Privacy-preserving**: The data is never published on-chain unless the user chooses to.

## Limitations

* **Probabilistic**: Without revealing all chunks, verification is never perfectly sound.
* **Requires tooling**: Verifiers need scripts or software to parse and verify proofs.
* **Data is not fully embedded onchain**: To that I say, grow up. Embrace your inner cypherpunk. Embrace probabilistic proofs.
