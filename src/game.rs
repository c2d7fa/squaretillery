#[derive(Debug, Clone, Copy)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

#[derive(Debug, Clone, Copy)]
pub struct Card {
    suit: Suit,
    value: u8,
}

#[derive(Debug)]
pub struct Pile {
    cards: Vec<Card>,  // NOTE: The top of the pile is at the back.
}

#[derive(Debug)]
pub struct Board {
    stacks: [[Option<Card>; 5]; 5],
}

#[derive(Debug)]
pub struct Game {
    drawn: Option<Card>,  // Last drawn card, currently waiting to be placed
    deck: Pile,           // Remaining cards not on the board
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

    pub fn is_royal(&self) -> bool {
        self.value >= 11
    }
}

impl Pile {
    pub fn new() -> Self {
        Self { cards: vec![] }
    }

    pub fn new_shuffled_deck() -> Self {
        use rand::seq::SliceRandom;

        let mut cards = Vec::new();
        for suit in vec![Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds] {   // TODO: Not sure if this is the best way to iterate over a static list of values.
            for value in 1..(13 + 1) {
                cards.push(Card::new(value, suit).unwrap());
            }
        }

        cards.shuffle(&mut rand::thread_rng());

        Self { cards }
    }

    pub fn top(&self) -> Option<Card> {
        self.cards.last().map(|card| { *card })
    }

    pub fn place_on_top(&mut self, card: Card) {
        self.cards.push(card);
    }

    // Note: This consumes the other pile.
    pub fn place_pile_on_top(&mut self, mut pile: Pile) {
        self.cards.append(&mut pile.cards);
    }

    pub fn draw(&mut self) -> Result<Card, String> {
        if self.cards.is_empty() { return Err("Tried to draw card when there was no more cards in the deck.".to_string()); }
        Ok(self.cards.pop().unwrap())
    }
}

#[derive(Clone, Copy)]
pub struct BoardPosition {
    x: i8,
    y: i8,
}

impl BoardPosition {
    pub fn new((x, y): (i8, i8)) -> Result<BoardPosition, String> {
        if x < -2 { return Err(format!("{:?} is an invalid coordiate: x coordinate {} is too low (must be between -2 and 2).", (x, y), x)) };
        if x >  2 { return Err(format!("{:?} is an invalid coordiate: x coordinate {} is too high (must be between -2 and 2).", (x, y), x)) };
        if y < -2 { return Err(format!("{:?} is an invalid coordiate: y coordinate {} is too low (must be between -2 and 2).", (x, y), y)) };
        if y >  2 { return Err(format!("{:?} is an invalid coordiate: y coordinate {} is too high (must be between -2 and 2).", (x, y), y)) };
        if (x == -2 || x == 2) && (y == -2 || y == 2) { return Err(format!("{:?} is an invalid coordianate: there are no cards in the corners of the board.", (x, y))); }

        Ok(BoardPosition { x, y })
    }

    pub fn x(&self) -> i8 { self.x }
    pub fn y(&self) -> i8 { self.y }
}

impl Board {
    pub fn new_empty() -> Board {
        Board { stacks: [[None; 5]; 5] }
    }

    pub fn get_card_at(&self, pos: BoardPosition) -> Option<Card> {
        self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize]
    }

    pub fn place_card_at(&mut self, pos: BoardPosition, card: Card) {
        self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize] = Some(card);
    }

    pub fn remove_card_at(&mut self, pos: BoardPosition) {
        self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize] = None;
    }
}

impl Game {
    pub fn new() -> Game {
        Game { drawn: None, deck: Pile::new_shuffled_deck(), board: Board::new_empty() }
    }

    pub fn set_up(&mut self) {
        let mut royals_pile = Pile::new();

        for position in vec![(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            'search_card: loop {
                let card = self.deck.draw().unwrap();
                if card.is_royal() {
                    royals_pile.place_on_top(card);
                } else {
                    self.board.place_card_at(BoardPosition::new(position).unwrap(), card);
                    break 'search_card;
                }
            }
        }

        self.deck.place_pile_on_top(royals_pile);
    }

    pub fn get_card_at(&self, pos: BoardPosition) -> Option<Card> {
        self.board.get_card_at(pos)
    }

    pub fn place_card_at(&mut self, pos: BoardPosition) -> Result<(), String> {
        if self.drawn.is_none() {
            Err("Cannot place card because there is no drawn card.".to_string())
        } else {
            self.board.place_card_at(pos, self.drawn.unwrap());
            self.drawn = None;
            Ok(())
        }
    }

    // TODO: Temporary. Game should know when to remove cards itself.
    pub fn remove_card_at(&mut self, pos: BoardPosition) {
        self.board.remove_card_at(pos);
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

