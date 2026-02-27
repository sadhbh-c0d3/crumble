# Crumble (CRyptographic gaMBLE) ♠️♥️♦️♣️ 

**Mental Poker** implemented using **Boneh–Lynn–Shacham (BLS)** cryptography.

**Author:** [Sonia Code](https://github.com/sadhbh-c0d3)

*Copyright (c) 2026 Sonia Code; See [LICENSE](LICENSE) file for license details.*

---

## The 1979 Problem: Mental Poker 🂮🂮🂮🂮🂮 🂮🂮 🂮🂮

In 1979, Adi Shamir, Ronald Rivest, and Leonard Adleman (the "RSA" trio) published **"Mental Poker,"** a paper posing a radical question:

> *Is it possible to play a fair game of poker over a communications channel without a trusted third party?*

For decades, this remained a notoriously difficult problem to solve at scale. Traditional online poker relies on a central server (the "House") to shuffle the deck. This creates a single point of failure and requires players to blindly trust that the House isn't rigging the deck or peeking at hole cards. While there have been attempts at implementing Mental Poker in Rust, they often struggled with heavy computational overhead or cryptographic side-channel vulnerabilities.

## The Crumble Solution 🂡🂢🂣🂮🂮 🂮🂮 🂮🂮

**Crumble** is a novel, modern realization of this 47-year-old dream, optimized for the next generation of decentralized finance. By utilizing **Boneh–Lynn–Shacham (BLS)** signatures and their unique **Bilinear Pairing** property, Crumble removes the need for a central dealer entirely.

This implementation shatters the centralized "House" model and rebuilds it entirely on a foundation of pure math.

* ♠ **Sovereign Shuffling:** Players sequentially "lock" and shuffle the deck using ephemeral keys. No single player knows the order of the cards, yet everyone can mathematically verify the deck's integrity.
* ♥️ **Out-of-Order Peeling:** Because of the BLS pairing properties, the cryptographic layers applied by the players can be "peeled" off in any order, allowing specific community and hole cards to be revealed safely.
* ♣️ **The Sovereign Referee Protocol:** Instead of a "House" that takes a rake, the blockchain acts as a *Stateless Referee.* It only intervenes if a player is caught cheating through a fraud proof, ensuring the game remains fast, cheap, and truly private.

## The $O(M)$ Breakthrough 🂡🂢🂣🂪🂮 🂮🂮 🂮🂮

Standard cryptographic shuffles are notorious gas-guzzlers. Verifying a shuffle on-chain usually requires $O(N^2)$ operations, which immediately hits the block gas limit.

Crumble bypasses this bottleneck using an optimized **Shuffle Trace**. Instead of forcing the smart contract to verify the entire 52-card matrix, the protocol only runs the heavy Miller Loop audits on the $M$ cards that are actually unmasked during the game. This reduces the verification complexity to $O(M)$, leveraging the efficiency of the Miller Loop for pairing-based checks.

## Architecture Agnosticism 🂡🂢🂣🂪🂨 🂮🂮 🂮🂮

Because the core cryptography and the state machine are completely decoupled from the networking layer, Crumble's design is highly flexible:

* **Arbitrum Orbit Chains (Stylus & Rust):** The game logic can be compiled directly to WebAssembly and deployed. The L3 chain acts as a fully decentralized, unstoppable execution layer.
* **Completely Off-Chain Poker:** Players can execute the entire game off-chain via a P2P network. The blockchain is only touched for final state submission and verifying fraud proofs, reducing normal gameplay gas costs to zero.
* **Trustless Client-Server:** The protocol can be deployed using a traditional central server for ultra-low latency. Because the deck is secured by BLS masking, the server simply acts as a message router—it is mathematically incapable of seeing the hole cards or rigging the deck.

## Implementation Review: The Bot Loop 🂡🂢🂣🂪🂨 🂴🂦 🂮🂮

The `PokerBot` logic in `main.rs` demonstrates the elegance of **Selective Unmasking**. When it is time to reveal hole cards, the engine executes the following:

```rust
PokerHandStateEnum::UnmaskHoleCards { player } => {
    let mut cards = hand.get_player_cards().clone();
    for i in 0..cards.len() {
        if i != player {
            cards[i].unmask(self.sk); // "I peel my layer off YOUR cards, but keep mine locked."
        }
    }
    hand.submit_player_cards(player, cards)
}

```

This captures the "Peeling" phase of the protocol. By unmasking everyone else's cards but their own, the bot ensures that the final cryptographic lock on any given card can only be opened by its rightful owner.

## Quickstart 🂡🂢🂣🂪🂨 🂴🂦 🂴🃂

To see the cryptographic engine in action, you can run the local simulation. This spins up the bots, performs the BLS shuffle, executes the betting rounds, and runs the final unmasking verification traces.

```bash
cargo run -p crum_bot --bin crum_bot

```

## Acknowledgements 🃏

Thanks to AI technologies for serving as a sounding board, accelerating the testing of these cryptographic concepts and state machine designs.
