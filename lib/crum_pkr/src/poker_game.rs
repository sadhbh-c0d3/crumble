/// Sovereign Referee Protocol (SRP) - Core Cryptographic Kernel
/// Designed by the Sonia-Code & Gemini (2026)
/// Foundation: Mental Poker (1979) -> Arbitrum Stylus (2026)

use crum_bls::types::PublicKey;

use crate::poker_deck::{MaskedCards, PokerDeck, UnmaskedCards};

pub struct PokerTable {
    max_players: usize,
    max_rounds: usize,
    current_players: Vec<u32>,
    dealer_button: usize,
    current_hand: Option<PokerHand>,
}

impl PokerTable {
    /// Player 1 creates a table
    pub fn new(max_players: usize, max_rounds: usize) -> Self {
        Self {
            max_players,
            max_rounds,
            current_players: vec![],
            dealer_button: 0,
            current_hand: None,
        }
    }

    /// Player 1, 2 (3,4,...) joins a table
    pub fn join(&mut self, player: u32) {
        // check player already joined
        self.current_players.push(player);
        // emit player joined
    }

    /// Player 1 starts new hand (at their discretion) with players at the table
    pub fn start(&mut self) {
        // check player 1 is submitter
        // check hand in progress
        self.current_hand.replace(PokerHand::new(
            self.current_players.len(),
            self.max_rounds,
            self.dealer_button,
        ));
        // emit hand started
    }

    /// Supports gameplay
    pub const fn get_current_hand(&self) -> Option<&PokerHand> {
        self.current_hand.as_ref()
    }

    /// Supports gameplay
    pub const fn get_current_hand_mut(&mut self) -> Option<&mut PokerHand> {
        self.current_hand.as_mut()
    }

    pub const fn get_max_players(&self) -> usize {
        self.max_players
    }

    pub const fn get_max_rounds(&self) -> usize {
        self.max_rounds
    }

    pub const fn get_current_player_count(&self) -> usize {
        self.current_players.len()
    }

    pub fn get_player(&self, player: usize) -> Option<u32> {
        self.current_players.get(player).cloned()
    }
}

pub const POKER_HAND_STATE_SHUFFLE: u8 = 0;
pub const POKER_HAND_STATE_SMALL_BLIND: u8 = 1;
pub const POKER_HAND_STATE_BIG_BLIND: u8 = 2;
pub const POKER_HAND_STATE_BET: u8 = 3;
pub const POKER_HAND_STATE_UNMASK_HOLE_CARDS: u8 = 4;
pub const POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS: u8 = 5;
pub const POKER_HAND_STATE_UNMASK_SHOWDOWN: u8 = 6;
pub const POKER_HAND_STATE_SUBMIT_PUBLIC_KEY: u8 = 7;
pub const POKER_HAND_STATE_FINISHED: u8 = 8;

pub const POKER_HOLDEM_PREFLOP: usize = 0;
pub const POKER_HOLDEM_FLOP: usize = 1;
pub const POKER_HOLDEM_TURN: usize = 2;
pub const POKER_HOLDEM_RIVER: usize = 3;
pub const POKER_HOLDEM_ROUNDS: usize = 4;

pub enum PokerHandStateEnum {
    Shuffle { player: usize },
    SmallBlind { player: usize },
    BigBlind { player: usize },
    Bet { round: usize, player: usize },
    UnmaskHoleCards { player: usize },
    UnmaskCommunityCards { round: usize, player: usize },
    UnmaskShowdown { player: usize },
    SubmitPublicKey { player: usize },
    Finished,
    Invalid,
}

pub struct PokerHandState {
    dealer_button: usize,
    num_players: usize,
    max_rounds: usize,
    current_player: usize,
    current_round: usize,
    current_state: u8,
}

impl PokerHandState {
    pub const fn new(num_players: usize, max_rounds: usize, dealer_button: usize) -> Self {
        Self {
            num_players,
            max_rounds,
            dealer_button,
            current_player: dealer_button,
            current_round: 0,
            current_state: POKER_HAND_STATE_SHUFFLE,
        }
    }

    pub fn next_dealer(&mut self) {
        self.current_player = self.dealer_button;
    }

    pub fn next_player(&mut self) -> bool {
        self.current_player = (self.current_player + 1) % self.num_players;
        self.current_player == self.dealer_button
    }

    pub fn next_round(&mut self) -> Result<bool, Vec<u8>> {
        let next_round = self.current_round + 1;

        if next_round > self.max_rounds {
            return Err(b"No next round - Hand has finished")?;
        }

        self.current_round = next_round;

        if next_round == self.max_rounds {
            return Ok(true);
        }

        Ok(false)
    }

    pub const fn to_tuple(&self) -> (usize, usize, u8) {
        (self.current_round, self.current_player, self.current_state)
    }

    pub const fn to_enum(&self) -> PokerHandStateEnum {
        match self.current_state {
            POKER_HAND_STATE_SHUFFLE => PokerHandStateEnum::Shuffle {
                player: self.current_player,
            },
            POKER_HAND_STATE_SMALL_BLIND => PokerHandStateEnum::SmallBlind {
                player: self.current_player,
            },
            POKER_HAND_STATE_BIG_BLIND => PokerHandStateEnum::BigBlind {
                player: self.current_player,
            },
            POKER_HAND_STATE_BET => PokerHandStateEnum::Bet {
                round: self.current_round,
                player: self.current_player,
            },
            POKER_HAND_STATE_UNMASK_HOLE_CARDS => PokerHandStateEnum::UnmaskHoleCards {
                player: self.current_player,
            },
            POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS => PokerHandStateEnum::UnmaskCommunityCards {
                round: self.current_round,
                player: self.current_player,
            },
            POKER_HAND_STATE_UNMASK_SHOWDOWN => PokerHandStateEnum::UnmaskShowdown {
                player: self.current_player,
            },
            POKER_HAND_STATE_SUBMIT_PUBLIC_KEY => PokerHandStateEnum::SubmitPublicKey {
                player: self.current_player,
            },
            POKER_HAND_STATE_FINISHED => PokerHandStateEnum::Finished,
            _ => PokerHandStateEnum::Invalid,
        }
    }
}

pub struct PokerHand {
    /// player_keys[public keys]
    poker_deck: PokerDeck,
    shuffled_deck: MaskedCards,
    player_cards: Vec<UnmaskedCards>,
    player_keys: Vec<Option<PublicKey>>,
    community_cards: Vec<UnmaskedCards>,
    current_state: PokerHandState,
}

impl PokerHand {
    pub fn new(num_players: usize, max_rounds: usize, dealer_button: usize) -> Self {
        let poker_deck = PokerDeck::new();
        let shuffled_deck = poker_deck.masked_cards();
        Self {
            poker_deck,
            shuffled_deck,
            player_cards: (0..num_players).map(|_| UnmaskedCards::default()).collect(),
            player_keys: (0..num_players).map(|_| None).collect(),
            community_cards: (0..max_rounds).map(|_| UnmaskedCards::default()).collect(),
            current_state: PokerHandState::new(num_players, max_rounds, dealer_button),
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
        self.community_cards.get(round - 1)
    }

    /// Called by each player to submit shuffled and masked deck
    pub fn submit_shuffled_deck(
        &mut self,
        player: usize,
        deck: MaskedCards,
    ) -> Result<(), Vec<u8>> {
        // check current player is submitter

        let PokerHandStateEnum::Shuffle { player: p } = self.get_current_state().to_enum() else {
            return Err(b"Not in shuffle state")?;
        };

        if p != player {
            return Err(b"Not your turn to shuffle")?;
        }

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

        for cards in self.player_cards.iter_mut() {
            *cards = self.shuffled_deck.deal(2);
        }

        self.current_state.next_player();
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

        self.player_cards = player_cards;

        // emit player cards unmasked by player

        if self.current_state.next_player() {
            self.current_state.current_state = POKER_HAND_STATE_BET;
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
        *round_cards = cards;

        // emit community cards for round unmasked by player

        if self.current_state.next_player() {
            self.current_state.current_state = POKER_HAND_STATE_BET;
        }

        Ok(())
    }

    /// Called at the end of hand to verify faierness of gameplay
    pub fn submit_public_key(&mut self, player: usize, pk: PublicKey) -> Result<(), Vec<u8>> {
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

        if self.current_state.next_player() {
            // TODO
            // verify deck using public keys
            // compute score of each hand
            // select winner
            self.current_state.current_state = POKER_HAND_STATE_FINISHED;
        }

        Ok(())
    }

    pub fn submit_bet(&mut self, player: usize) -> Result<(), Vec<u8>> {
        let PokerHandStateEnum::Bet {
            round: r,
            player: p,
        } = self.get_current_state().to_enum()
        else {
            return Err(b"Not in bet state")?;
        };

        if p != player {
            return Err(b"Not your turn to bet")?;
        }

        // TODO: implement proper Poker betting logic, progress to next round
        // based on called bets. Here we just test cryptographic cards masking.
        if self.current_state.next_player() {
            if self.current_state.next_round()? {
                self.current_state.current_state = POKER_HAND_STATE_UNMASK_SHOWDOWN;
            } else {
                let num_cards_deal = if r == POKER_HOLDEM_PREFLOP { 3 } else { 1 };
                self.community_cards[r] = self.shuffled_deck.deal(num_cards_deal);
                self.current_state.current_state = POKER_HAND_STATE_UNMASK_COMMUNITY_CARDS;
            }
        }

        Ok(())
    }
}
