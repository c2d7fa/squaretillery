#[derive(Debug, Clone, Copy)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

#[derive(Debug, Clone, Copy)] // TODO: Do we want Copy?
pub struct Card {
    suit: Suit,
    value: u8,
}

pub struct Deck {
    cards: Vec<Card>,  // NOTE: The top of the deck is at the back.
}

pub struct Board {
    stacks: [[Option<Card>; 5]; 5],
}

pub struct Game {
    drawn: Option<Card>,  // Last drawn card, currently waiting to be placed
    deck: Deck,           // Remaining cards not on the board
    board: Board,
}

// TODO: Jokers
impl Card {
    pub fn new(value: u8, suit: Suit) -> Result<Card, String> {
        if !(value >= 1 && value <= 13) { return Err(format!("{} is not a valid value for card (must be between 1 and 13).", value)) };

        Ok(Card { suit, value })
    }

    pub fn format_text_short(&self) -> String {
        let suit_string = match &self.suit {
            Suit::Spades => "S",
            Suit::Hearts => "H",
            Suit::Clubs => "C",
            Suit::Diamonds => "D",
        };
        format!("{}{}", suit_string, &self.value)
    }
}

impl Deck {
    pub fn new_shuffled() -> Deck {
        use rand::seq::SliceRandom;

        let mut cards = Vec::new();
        for suit in vec![Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {   // TODO: Not sure if this is the best way to iterate over a static list of values.
            for value in 1..(13 + 1) {
                cards.push(Card::new(value, suit).unwrap());
            }
        }

        cards.shuffle(&mut rand::thread_rng());

        Deck { cards }
    }

    pub fn draw(&mut self) -> Result<Card, String> {
        if self.cards.is_empty() { return Err("Tried to draw card when there was no more cards in the deck.".to_string()); }
        Ok(self.cards.pop().unwrap())
    }
}


impl Board {
    pub fn new_empty() -> Board {
        Board { stacks: [[None; 5]; 5] }
    }

    pub fn get_card_at(&self, x: i8, y: i8) -> Result<Option<Card>, String> {
        Self::check_coord(x, y).map(|(x, y)| {
            return self.stacks[(2 + x) as usize][(2 + y) as usize];
        })
    }

    pub fn place_card_at(&mut self, x: i8, y: i8, card: Card) -> Result<(), String> {
        Self::check_coord(x, y).map(|(x, y)| {
            self.stacks[(2 + x) as usize][(2 + y) as usize] = Some(card);
        })
    }

    pub fn remove_card_at(&mut self, x: i8, y: i8) -> Result<(), String> {
        Self::check_coord(x, y).map(|(x, y)| {
            self.stacks[(2 + x) as usize][(2 + y) as usize] = None;
        })
    }

    fn check_coord(x: i8, y: i8) -> Result<(i8, i8), String> {
        if x < -2 { return Err(format!("{:?} is an invalid coordiate: x coordinate {} is too low (must be between -2 and 2).", (x, y), x)) };
        if x >  2 { return Err(format!("{:?} is an invalid coordiate: x coordinate {} is too high (must be between -2 and 2).", (x, y), x)) };
        if y < -2 { return Err(format!("{:?} is an invalid coordiate: y coordinate {} is too low (must be between -2 and 2).", (x, y), y)) };
        if y >  2 { return Err(format!("{:?} is an invalid coordiate: y coordinate {} is too high (must be between -2 and 2).", (x, y), y)) };
        if (x == -2 || x == 2) && (y == -2 || y == 2) { return Err(format!("{:?} is an invalid coordianate: there are no cards in the corners of the board.", (x, y))); }

        Ok((x, y))
    }
}

impl Game {
    pub fn new() -> Game {
        Game { drawn: None, deck: Deck::new_shuffled(), board: Board::new_empty() }
    }

    pub fn get_card_at(&self, x: i8, y: i8) -> Result<Option<Card>, String> {
        self.board.get_card_at(x, y)
    }

    pub fn place_card_at(&mut self, x: i8, y: i8) -> Result<(), String> {
        if self.drawn.is_none() {
            Err("Cannot place card because there is no drawn card.".to_string())
        } else {
            self.board.place_card_at(x, y, self.drawn.unwrap())?;
            self.drawn = None;
            Ok(())
        }
    }

    pub fn draw(&mut self) -> Result<(), String> {
        if self.drawn.is_some() { return Err("Cannot draw card while one is already drawn.".to_string()) }
        self.deck.draw().map(|card| {
            self.drawn = Some(card);
        })
    }

    pub fn drawn(&self) -> Option<Card> {
        self.drawn
    }
}

