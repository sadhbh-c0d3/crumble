# Crumble (CRyptographic gaMBLE)

**Mental Poker** implemented using **Boneh–Lynn–Shacham (BLS)** cryptography.

## About the Origins: The Unsolved Problem of Mental Poker

In 1979, Adi Shamir, Ronald Rivest, and Leonard Adleman (the "RSA" trio) published **"Mental Poker,"** a paper posing a radical question: *Is it possible to play a fair game of poker over a communications channel without a trusted third party?*

For decades, this remained a notoriously difficult problem to solve at scale. Traditional online poker relies on a central server (the "House") to shuffle the deck. This creates a single point of failure and requires players to blindly trust that the House isn't rigging the deck or peeking at hole cards. While there have been attempts at implementing Mental Poker in Rust, they often struggled with heavy computational overhead or cryptographic side-channel vulnerabilities.

## The Crumble Solution

**Crumble (a portmanteau of CRyptographic gaMBLE) is a novel, modern realization of this 47-year-old dream, optimized for the next generation of decentralized finance.** By utilizing **Boneh–Lynn–Shacham (BLS)** signatures and their unique **commutative masking** property, Crumble removes the need for a central dealer entirely. This implementation stands as possibly the very first attempt to solve the Mental Poker problem in Rust using BLS cryptography, shattering the centralized "House" model and rebuilding it entirely on a foundation of **pure math**.

* **Sovereign Shuffling**: Players sequentially "lock" and shuffle the deck using ephemeral keys. No single player knows the order of the cards, yet everyone can verify the deck's integrity.
* **Arbitrum Stylus Ready**: While currently a pure Rust implementation, Crumble is architected for **Arbitrum Stylus / Rust**. This allows the heavy cryptographic lifting (the Miller Loop audits) to run at near-native speeds on an L3 Orbit chain.
* **The Sovereign Referee Protocol**: Instead of a "House" that takes a rake, the blockchain acts as a ***"Stateless Referee."*** It only intervenes if a player is caught cheating through a fraud proof, ensuring the game remains fast, cheap, and truly private.

## Technical Features & Architecture Agnosticism

Because the core cryptography and the state machine are completely decoupled from the networking layer, Crumble's design is highly flexible. The `PokerHandState` engine supports multiple deployment models without rewriting the core rules:

* **Arbitrum Orbit Chains (Stylus & Rust)**: The game logic can be compiled directly to WebAssembly and deployed as an Arbitrum Stylus smart contract. The L3 chain acts as a fully decentralized, unstoppable execution layer for the Sovereign Referee.
* **Completely Off-Chain Poker**: Players can execute the entire game off-chain via a P2P network or lightweight relay. The blockchain is only touched for the final state submission and verifying fraud proofs (running the Miller Loop). This reduces gas costs to zero during normal gameplay while retaining absolute cryptographic security.
* **Traditional Client-Server (Replication Friendly)**: The protocol can be deployed using a traditional central server for ultra-low latency game coordination. Because the deck is secured by commutative masking, the server simply acts as a message router. It is mathematically incapable of seeing the hole cards or rigging the deck, making it incredibly replication-friendly for massive player bases while removing user trust requirements.

## Implementation Review: The Bot Loop

The `PokerBot` logic in `main.rs` is particularly elegant because of how it handles **Selective Unmasking**:

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

This perfectly captures the "Peeling" phase of the protocol. By unmasking everyone else's cards but their own, the bot ensures that the final "peel" is always the owner's choice.

## Acknowledgements

Thanks to AI technologies, I was able to consult my ideas with AI, enhancing my knowledge and accelerating the testing of my ideas.
