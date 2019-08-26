#[derive(Debug)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

#[derive(Debug)]
pub struct Card {
    suit: Suit,
    value: u8,
}

pub struct Deck {
}

pub struct Board {
}

pub struct Game {
    deck: Deck,    // Remaining cards not on the board
    board: Board,
}

impl Card {
    pub fn new(value: u8, suit: Suit) -> Result<Card, String> {
        if !(value >= 1 && value <= 13) { return Err(format!("{} is not a valid value for card (must be between 1 and 13).", value)) };
        return Ok(Card { suit, value });
    }

    pub fn format_text_short(&self) -> String {
        let suit_string = match &self.suit {
            Suit::Spades => "S",
            Suit::Hearts => "H",
            Suit::Clubs => "C",
            Suit::Diamonds => "D",
        };
        return format!("{}{}", suit_string, &self.value);
    }
}

fn check_coord(x: i8, y: i8) -> Result<(i8, i8), String> {
    if x < -2 { return Err(format!("x coordinate {} is too low (must be between -2 and 2).", x)) };
    if x >  2 { return Err(format!("x coordinate {} is too high (must be between -2 and 2).", x)) };
    if y < -2 { return Err(format!("y coordinate {} is too low (must be between -2 and 2).", y)) };
    if y >  2 { return Err(format!("y coordinate {} is too high (must be between -2 and 2).", y)) };
    if (x == -2 || x == 2) && (y == -2 || y == 2) { return Err(format!("{:?} is not a valid coordinate (there are no cards in the corners of the board).", (x, y))); }
    return Ok((x, y));
}

impl Board {
    pub fn get_card_at(&self, x: i8, y: i8) -> Result<Option<Card>, String> {
        check_coord(x, y).map(|(x, y)| {
            if (x, y) == (0, 0) { return None }
            else if x == -2 || y == -2 || x == 2 || y == 2 { return Some(Card::new(11, Suit::Spades).unwrap()) }
            else { return Some(Card::new(5, Suit::Hearts).unwrap()) }
        })
    }
}

impl Game {
    pub fn new() -> Game {
        return Game {deck: Deck {}, board: Board {}};
    }

    pub fn get_card_at(&self, x: i8, y: i8) -> Result<Option<Card>, String> {
        return self.board.get_card_at(x, y);
    }
}

