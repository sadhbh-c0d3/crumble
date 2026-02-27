//! Crumble (CRyptographic gaMBLE)
//! 
//! Mental Poker (1979) implemented using Boneh–Lynn–Shacham (BLS) cryptography.
//! Designed by the Sonia Code & Gemini AI (2026)
//! 
//! Copyright (c) 2026 Sonia Code; See LICENSE file for license details.

use bls12_381::Scalar;
use crum_bls::{types::SigningKey, util::make_public_key_from_signing_key, verify};
use crum_pkr::{
    poker_deck::PokerCard,
    poker_state::{POKER_HOLDEM_ROUNDS, PokerHandStateEnum},
    poker_table::PokerTable,
};
use ff::Field;
use itertools::Itertools;
// use rand::{Rng, distributions::Uniform, rngs::ThreadRng, thread_rng};
use rand::{
    Rng,
    distributions::{Uniform, WeightedIndex},
    rngs::ThreadRng,
    thread_rng,
};

pub struct PokerCards(Vec<Option<PokerCard>>);

impl ToString for PokerCards {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|opt_c| opt_c.as_ref().map_or("_!".to_string(), |c| c.to_string()))
            .join(", ")
    }
}

pub struct PokerBot {
    player_id: u32,
    rng: ThreadRng,
    sk: SigningKey,
    shuffle_trace: Option<Vec<verify::ShuffleTrace>>,
}

impl PokerBot {
    pub fn new(player_id: u32) -> Self {
        let mut rng = thread_rng();
        let sk = Scalar::random(&mut rng);
        Self {
            player_id,
            rng,
            sk,
            shuffle_trace: None,
        }
    }

    pub fn act(&mut self, poker_table: &mut PokerTable) -> Result<(), Vec<u8>> {
        let Some(hand) = poker_table.get_current_hand_mut() else {
            return Err(b"No active hand to act upon")?;
        };

        let poker_state = hand.get_current_state().to_enum();

        match poker_state {
            PokerHandStateEnum::Shuffle { player, is_dealer } => {
                tracing::info!("Shuffle on Player {} (is_dealer={})", player + 1, is_dealer);
                let mut cards = if is_dealer {
                    hand.get_poker_deck().masked_cards()
                } else {
                    hand.get_shuffled_deck().clone()
                };
                cards.mask(self.sk);
                self.shuffle_trace
                    .replace(cards.shuffle_traced(&mut self.rng));
                hand.submit_shuffled_deck(player, cards)?;
                Ok(())
            }
            PokerHandStateEnum::SmallBlind { player } => {
                tracing::info!("Small Blind on Player {}", player + 1);
                hand.submit_small_blind(player)
            }
            PokerHandStateEnum::BigBlind { player } => {
                tracing::info!("Big Blind on Player {}", player + 1);
                hand.submit_big_blind(player)
            }
            PokerHandStateEnum::Bet { round, player } => {
                let min_bet = hand.get_call_amount_required(player)?;
                let small_blind = hand.get_small_blind();
                let chips = hand.get_chips_remaining(player);
                let bet = if chips < min_bet {
                    0
                } else {
                    let weights = [1, 4, 8];
                    let dist = WeightedIndex::new(&weights)
                        .or_else(|_| Err(b"Failed to create weighted index"))?;
                    let action = self.rng.sample(dist);
                    match action {
                        0 => 0,
                        1 => min_bet,
                        _ => {
                            let start_unit = (min_bet + small_blind - 1) / small_blind;
                            let end_unit = chips / small_blind;
                            if start_unit <= end_unit {
                                let units = self
                                    .rng
                                    .sample(Uniform::new_inclusive(start_unit, end_unit.min(10)));
                                units * small_blind
                            } else {
                                min_bet
                            }
                        }
                    }
                };
                tracing::info!(
                    "Round {} Bet on Player {} (${})",
                    round + 1,
                    player + 1,
                    bet
                );
                hand.submit_bet(player, bet)
            }
            PokerHandStateEnum::UnmaskHoleCards { player } => {
                tracing::info!("Unmask Hole Cards on Player {}", player + 1);
                let mut cards = hand.get_player_cards().clone();
                for i in 0..cards.len() {
                    if i != player {
                        cards[i].unmask(self.sk);
                    }
                }
                hand.submit_player_cards(player, cards)
            }
            PokerHandStateEnum::UnmaskCommunityCards { round, player } => {
                tracing::info!(
                    "Round {} Unmask Community Cards on Player {}",
                    round + 1,
                    player + 1
                );
                let Some(mut cards) = hand.get_community_cards(round).cloned() else {
                    return Err(b"No community cards for round")?;
                };
                cards.unmask(self.sk);
                hand.submit_community_cards(player, round, cards)
            }
            PokerHandStateEnum::UnmaskShowdown { player } => {
                tracing::info!("Unmask Showdown on Player {}", player + 1);
                let mut cards = hand.get_player_cards().clone();
                if cards.get_mut(player).map(|c| c.unmask(self.sk)).is_none() {
                    return Err(b"Invalid player cards for showdown")?;
                }
                hand.submit_player_cards_showdown(player, cards)
            }
            PokerHandStateEnum::SubmitPublicKey { player } => {
                tracing::info!("Submit Public Key on Player {}", player + 1);
                let pk = make_public_key_from_signing_key(&self.sk);
                let Some(shuffle_trace) = self.shuffle_trace.take() else {
                    return Err(b"No shuffle trace")?;
                };
                hand.submit_public_key(player, pk, shuffle_trace)
            }
            PokerHandStateEnum::Finished => {
                tracing::info!("Hand is finished");
                Ok(())
            }
            PokerHandStateEnum::Cheated { player } => {
                tracing::info!("Cheated by Player {}", player + 1);
                Err(b"Player cheated")?
            }
            PokerHandStateEnum::Invalid => Err(b"Invalid poker state")?,
        }
    }
}

pub fn run(num_players: usize, inital_chips: u64, small_blind: u64) -> Result<(), Vec<u8>> {
    let mut bots: Vec<_> = (0..num_players)
        .map(|i| PokerBot::new(1u32 + (i as u32)))
        .collect();

    let mut poker_table = PokerTable::new(num_players, POKER_HOLDEM_ROUNDS);

    bots.iter().for_each(|b| poker_table.join(b.player_id));
    poker_table.start_hand(inital_chips, small_blind)?;

    loop {
        let Some(hand) = poker_table.get_current_hand() else {
            return Err(b"Hand not started")?;
        };

        let state = hand.get_current_state();
        if state.is_finished() {
            let mut community_cards = Vec::new();
            for i in 0..POKER_HOLDEM_ROUNDS {
                if let Some(cards) = hand.get_community_cards(i) {
                    let cards = hand.get_poker_deck().unmasked_cards(cards);
                    community_cards.extend(cards);
                }
            }
            let community_cards_str = PokerCards(community_cards).to_string();
            tracing::info!("Community cards: {}", community_cards_str);

            let cards = hand.get_player_cards();
            for i in 0..num_players {
                let cards = hand.get_poker_deck().unmasked_cards(&cards[i]);
                let player_cards_str = PokerCards(cards).to_string();
                tracing::info!("Player {} cards: {}", i + 1, player_cards_str)
            }
            tracing::info!("Hand ended");
            break;
        }

        let player = state.get_current_player();
        let Some(player_id) = poker_table.get_player(player) else {
            return Err(b"Invalid player to act")?;
        };

        let Some(bot_index) = bots.iter().position(|b| b.player_id.eq(&player_id)) else {
            return Err(b"Bot player not found")?;
        };

        let Some(bot) = bots.get_mut(bot_index) else {
            return Err(b"Invalid bot player")?;
        };

        bot.act(&mut poker_table)?;
    }

    Ok(())
}

pub fn main() {
    tracing_subscriber::fmt::init();

    let mut rng = thread_rng();
    let num_players = rng.sample(Uniform::new_inclusive(2usize, 6usize));
    // let num_players = 6;
    let initial_chips = 1000;
    let small_blind = 10;

    if let Err(err) = run(num_players, initial_chips, small_blind) {
        let err_text = String::from_utf8(err).unwrap();
        tracing::error!("Error: {}", err_text);
    }
}
