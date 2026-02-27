/// Sovereign Referee Protocol (SRP) - Core Cryptographic Kernel
/// Designed by the Sonia-Code & Gemini (2026)
/// Foundation: Mental Poker (1979) -> Arbitrum Stylus (2026)
use crum_bls::{types::PublicKey, verify};

use crate::{
    poker_bets::PokerBettingState,
    poker_deck::{MaskedCards, PokerDeck, UnmaskedCards},
    poker_state::{
        POKER_HAND_STATE_BET, POKER_HAND_STATE_BIG_BLIND, POKER_HAND_STATE_CHEATED,
        POKER_HAND_STATE_FINISHED, POKER_HAND_STATE_SMALL_BLIND,
        POKER_HAND_STATE_SUBMIT_PUBLIC_KEY, POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS,
        POKER_HAND_STATE_UNMASK_HOLE_CARDS, POKER_HAND_STATE_UNMASK_SHOWDOWN, POKER_HOLDEM_PREFLOP,
        PokerHandState, PokerHandStateEnum,
    },
};

pub struct PokerHand {
    /// player_keys[public keys]
    poker_deck: PokerDeck,
    shuffled_deck: MaskedCards,
    shuffle_history: Vec<MaskedCards>,
    player_cards: Vec<UnmaskedCards>,
    player_keys: Vec<Option<PublicKey>>,
    community_cards: Vec<UnmaskedCards>,
    unmasking_sequence: Vec<(usize, u8, Vec<UnmaskedCards>)>,
    current_state: PokerHandState,
    betting_state: PokerBettingState,
    small_blind: u64,
}

impl PokerHand {
    pub fn new(
        num_players: usize,
        max_rounds: usize,
        dealer_button: usize,
        initial_chips: u64,
        small_blind: u64,
    ) -> Self {
        let poker_deck = PokerDeck::new();
        let shuffled_deck = poker_deck.masked_cards();
        Self {
            poker_deck,
            shuffled_deck,
            shuffle_history: vec![],
            player_cards: (0..num_players).map(|_| UnmaskedCards::default()).collect(),
            player_keys: (0..num_players).map(|_| None).collect(),
            community_cards: (0..max_rounds).map(|_| UnmaskedCards::default()).collect(),
            unmasking_sequence: vec![],
            current_state: PokerHandState::new(num_players, max_rounds, dealer_button),
            betting_state: PokerBettingState::new(num_players, initial_chips),
            small_blind,
        }
    }

    /// On event acting player checks the current round to follow the rules
    /// Note: the Poker rounds are split into smaller rounds such as:
    /// Player 1 shuffles and submits, Player 2 shuffles submits, Player 1 blinds,
    /// Player 2 blids, Player 1 deals and unmasks for others, Player 2 unmasks for others
    /// Player 1 unmasks own, Player 2 unmasks own, Player 1 bets, Player 2 bets,
    /// Player 1 deals flop community cards and unmasks, Player 2 unmasks community cards,
    /// Player 1 bets, Player 2 bets, ... (and so on)
    pub const fn get_current_state(&self) -> &PokerHandState {
        &self.current_state
    }

    /// Poker deck is constant, but we ensure all players have same reference point
    pub const fn get_poker_deck(&self) -> &PokerDeck {
        &self.poker_deck
    }

    /// Supports Player shuffle and submit
    pub const fn get_shuffled_deck(&self) -> &MaskedCards {
        &self.shuffled_deck
    }

    /// Supports Player cards unmask
    pub fn get_player_cards(&self) -> &Vec<UnmaskedCards> {
        &self.player_cards
    }

    /// Supports community cards unmask
    pub fn get_community_cards(&self, round: usize) -> Option<&UnmaskedCards> {
        if round == POKER_HOLDEM_PREFLOP {
            return None;
        }
        self.community_cards.get(round - 1)
    }

    /// Tell amount required to call (minimum bet)
    pub fn get_call_amount_required(&self, player: usize) -> Result<u64, Vec<u8>> {
        self.betting_state.call_amount_required(player)
    }

    /// Tell amount of chips remaining
    pub fn get_chips_remaining(&self, player: usize) -> u64 {
        self.betting_state.chips_remaining(player)
    }

    /// Tell small blind amount
    pub fn get_small_blind(&self) -> u64 {
        self.small_blind
    }

    /// Tell big blind amount
    pub fn get_big_blind(&self) -> u64 {
        self.small_blind * 2
    }

    /// Called by each player to submit shuffled and masked deck
    pub fn submit_shuffled_deck(
        &mut self,
        player: usize,
        deck: MaskedCards,
    ) -> Result<(), Vec<u8>> {
        // check current player is submitter

        let PokerHandStateEnum::Shuffle {
            player: p,
            is_dealer: _,
        } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in shuffle state")?;
        };

        if p != player {
            return Err(b"Not your turn to shuffle")?;
        }

        self.shuffle_history.push(deck.clone());
        self.shuffled_deck = deck;

        // emit shuffle submitted

        if self.current_state.next_player() {
            self.current_state.current_state = POKER_HAND_STATE_SMALL_BLIND;
        }

        Ok(())
    }

    pub fn submit_small_blind(&mut self, player: usize) -> Result<(), Vec<u8>> {
        let PokerHandStateEnum::SmallBlind { player: p } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in small blind state")?;
        };

        if p != player {
            return Err(b"Not your turn to post small blind")?;
        }

        self.betting_state
            .process_action(player, self.get_small_blind())?;

        self.current_state.next_player();
        self.current_state.current_state = POKER_HAND_STATE_BIG_BLIND;

        Ok(())
    }

    pub fn submit_big_blind(&mut self, player: usize) -> Result<(), Vec<u8>> {
        let PokerHandStateEnum::BigBlind { player: p } = self.get_current_state().to_enum() else {
            return Err(b"Not in big blind state")?;
        };

        if p != player {
            return Err(b"Not your turn to post big blind")?;
        }

        self.betting_state
            .process_action(player, self.get_big_blind())?;

        for cards in self.player_cards.iter_mut() {
            *cards = self.shuffled_deck.deal(2);
        }

        self.current_state.next_dealer();
        self.current_state.current_state = POKER_HAND_STATE_UNMASK_HOLE_CARDS;

        Ok(())
    }

    /// Called by each player to unmask player hand
    pub fn submit_player_cards(
        &mut self,
        player: usize,
        player_cards: Vec<UnmaskedCards>,
    ) -> Result<(), Vec<u8>> {
        // check current player is submitter
        let PokerHandStateEnum::UnmaskHoleCards { player: p } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in unmask hole cards state")?;
        };

        if p != player {
            return Err(b"Not your turn to unmask hole cards")?;
        }

        if player_cards.len() != self.player_cards.len() {
            return Err(b"Incorrect length of player cards")?;
        }

        self.unmasking_sequence.push((
            player,
            POKER_HAND_STATE_UNMASK_HOLE_CARDS,
            player_cards.clone(),
        ));
        self.player_cards = player_cards;

        // emit player cards unmasked by player

        if self.current_state.next_player() {
            self.current_state
                .next_player_masked(self.betting_state.get_active_players(), true);
            self.betting_state.next_street();
            self.current_state.current_state = POKER_HAND_STATE_BET;

            self.check_betting_round_complete()?;
        }

        Ok(())
    }

    /// Called by each player to unmask player hand
    pub fn submit_player_cards_showdown(
        &mut self,
        player: usize,
        player_cards: Vec<UnmaskedCards>,
    ) -> Result<(), Vec<u8>> {
        // check current player is submitter
        let PokerHandStateEnum::UnmaskShowdown { player: p } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in unmask hole cards state")?;
        };

        if p != player {
            return Err(b"Not your turn to unmask hole cards")?;
        }

        if player_cards.len() != self.player_cards.len() {
            return Err(b"Incorrect length of player cards")?;
        }

        self.unmasking_sequence.push((
            player,
            POKER_HAND_STATE_UNMASK_SHOWDOWN,
            player_cards.clone(),
        ));
        self.player_cards = player_cards;

        // emit player cards unmasked by player

        if self.current_state.next_player() {
            self.current_state.current_state = POKER_HAND_STATE_SUBMIT_PUBLIC_KEY;
        }

        Ok(())
    }

    /// Called by each player to unmask community cards
    pub fn submit_community_cards(
        &mut self,
        player: usize,
        round: usize,
        cards: UnmaskedCards,
    ) -> Result<(), Vec<u8>> {
        // check current player is submitter
        let PokerHandStateEnum::UnmaskCommunityCards {
            round: r,
            player: p,
        } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in bet state")?;
        };

        if r != round {
            return Err(b"Not this round to unmask cards")?;
        }

        if p != player {
            return Err(b"Not your turn to bet")?;
        }

        let round_cards = self
            .community_cards
            .get_mut(round - 1)
            .expect("No round cards");

        self.unmasking_sequence.push((
            player,
            POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS,
            vec![cards.clone()],
        ));
        *round_cards = cards;

        // emit community cards for round unmasked by player

        if self.current_state.next_player() {
            self.current_state
                .next_player_masked(self.betting_state.get_active_players(), true);
            self.betting_state.next_street();
            self.current_state.current_state = POKER_HAND_STATE_BET;

            self.check_betting_round_complete()?;
        }

        Ok(())
    }

    /// Called at the end of hand to verify faierness of gameplay
    pub fn submit_public_key(
        &mut self,
        player: usize,
        pk: PublicKey,
        traces: Vec<verify::ShuffleTrace>,
    ) -> Result<(), Vec<u8>> {
        let PokerHandStateEnum::SubmitPublicKey { player: p } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in submit public key state")?;
        };

        if p != player {
            return Err(b"Not your turn to submit public key")?;
        }

        let player_key = self.player_keys.get_mut(player).expect("No player key");
        *player_key = Some(pk);

        // emit (ephemeral) public key submitted

        if !self.verify_shuffle(player, pk, traces) {
            self.current_state.current_state = POKER_HAND_STATE_CHEATED;
            return Err("Player cheated during shuffle")?;
        }

        if self.current_state.next_player() {
            match self.verify_unmasking() {
                Ok(None) => (),
                Ok(Some(cheater)) => {
                    self.current_state.current_state = POKER_HAND_STATE_CHEATED;
                    return Err(
                        format!("Player cheated during unmasking {}", cheater).into_bytes()
                    )?;
                }
                Err(err) => Err(err)?,
            }
            // TODO
            // compute score of each hand
            // select winner
            self.current_state.current_state = POKER_HAND_STATE_FINISHED;
        }

        Ok(())
    }

    pub fn verify_shuffle(
        &mut self,
        player: usize,
        pk: PublicKey,
        traces: Vec<verify::ShuffleTrace>,
    ) -> bool {
        let num_players = self.current_state.num_players;
        let dealer = self.current_state.dealer_button;

        let step_index = (player + num_players - dealer) % num_players;

        let next_cards = self.shuffle_history[step_index].cards();
        let prev_cards = if step_index == 0 {
            self.poker_deck.cards()
        } else {
            self.shuffle_history[step_index - 1].cards()
        };

        verify::verify_shuffle_traced(&prev_cards, &next_cards, &pk, &traces).is_ok()
    }

    pub fn verify_unmasking(&mut self) -> Result<Option<usize>, Vec<u8>> {
        // Reconstruct the initial dealt state from the final shuffled deck
        let final_shuffled_deck = self
            .shuffle_history
            .last()
            .ok_or_else(|| b"No shuffle history")?
            .cards();

        let num_players = self.current_state.num_players;

        let mut deck_idx = 0;

        // Trackers for the "current" state of cards as they get peeled
        // Hole cards: one Vec<G1Affine> (2 cards) per player
        let mut tracked_hole_cards: Vec<Vec<bls12_381::G1Affine>> = Vec::new();
        for _ in 0..num_players {
            tracked_hole_cards.push(final_shuffled_deck[deck_idx..deck_idx + 2].to_vec());
            deck_idx += 2;
        }

        // Community cards: stored by round (Flop=3, Turn=1, River=1)
        let mut tracked_community_cards: Vec<Vec<bls12_381::G1Affine>> = vec![
            final_shuffled_deck[deck_idx..deck_idx + 3].to_vec(), // Flop
            final_shuffled_deck[deck_idx + 3..deck_idx + 4].to_vec(), // Turn
            final_shuffled_deck[deck_idx + 4..deck_idx + 5].to_vec(), // River
        ];

        let mut comm_round_idx = 0;
        let mut comm_unmask_count = 0;

        // Replay history and verify every single peel
        for (action_player, state_type, submitted_cards) in &self.unmasking_sequence {
            let action_pk =
                self.player_keys[*action_player].ok_or_else(|| b"Missing PK for unmask audit")?;

            let action_pk_g2 = bls12_381::G2Affine::from(action_pk);

            match *state_type {
                POKER_HAND_STATE_UNMASK_HOLE_CARDS => {
                    for target_player in 0..num_players {
                        if target_player == *action_player {
                            continue;
                        }

                        // Unmasking everyone else's hole cards
                        let before = &tracked_hole_cards[target_player];
                        let after = submitted_cards[target_player].cards();

                        for (b, a) in before.iter().zip(after.iter()) {
                            if !verify::verify_unmasking(*b, *a, action_pk_g2) {
                                self.current_state.current_state = POKER_HAND_STATE_CHEATED;
                                return Ok(Some(*action_player));
                            }
                        }
                        tracked_hole_cards[target_player] = after;
                    }
                }
                POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS => {
                    // Unmasking the current round of community cards
                    let before = &tracked_community_cards[comm_round_idx];
                    let after = submitted_cards[0].cards();

                    for (b, a) in before.iter().zip(after.iter()) {
                        if !verify::verify_unmasking(*b, *a, action_pk_g2) {
                            self.current_state.current_state = POKER_HAND_STATE_CHEATED;
                            return Ok(Some(*action_player));
                        }
                    }
                    tracked_community_cards[comm_round_idx] = after;

                    comm_unmask_count += 1;
                    if comm_unmask_count == num_players {
                        comm_unmask_count = 0;
                        comm_round_idx += 1; // Advance to Turn, then River
                    }
                }
                POKER_HAND_STATE_UNMASK_SHOWDOWN => {
                    // Unmasking THEIR OWN hole cards
                    let target_player = *action_player;
                    let before = &tracked_hole_cards[target_player];
                    let after = submitted_cards[target_player].cards();

                    for (b, a) in before.iter().zip(after.iter()) {
                        if !verify::verify_unmasking(*b, *a, action_pk_g2) {
                            self.current_state.current_state = POKER_HAND_STATE_CHEATED;
                            return Ok(Some(*action_player));
                        }
                    }
                    tracked_hole_cards[target_player] = after;
                }
                _ => {}
            }
        }

        Ok(None)
    }

    pub fn submit_bet(&mut self, player: usize, amount: u64) -> Result<(), Vec<u8>> {
        let PokerHandStateEnum::Bet {
            round: _,
            player: p,
        } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in bet state")?;
        };

        if p != player {
            return Err(b"Not your turn to bet")?;
        }

        self.betting_state.process_action(player, amount)?;
        self.current_state
            .next_player_masked(self.betting_state.get_active_players(), false);

        self.check_betting_round_complete()?;

        Ok(())
    }

    fn check_betting_round_complete(&mut self) -> Result<(), Vec<u8>> {
        if self.betting_state.is_betting_round_complete() {
            self.current_state.next_dealer();
            let round = self.current_state.current_round;

            if self.current_state.next_round()? {
                self.current_state.current_state = POKER_HAND_STATE_UNMASK_SHOWDOWN;
            } else {
                let num_cards_deal = if round == POKER_HOLDEM_PREFLOP { 3 } else { 1 };
                self.community_cards[round] = self.shuffled_deck.deal(num_cards_deal);
                self.current_state.current_state = POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS;
            }
        }
        Ok(())
    }
}
