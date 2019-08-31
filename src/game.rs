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
    stacks: [[Pile; 5]; 5],
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

    pub fn value(&self) -> u8 { self.value }
    pub fn suit(&self) -> Suit { self.suit }
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

    pub fn place_card_on_top(&mut self, card: Card) {
        self.cards.push(card);
    }

    // Note: This consumes the other pile.
    pub fn place_pile_on_top(&mut self, mut pile: Pile) {
        self.cards.append(&mut pile.cards);
    }

    // Note: This consumes the other pile.
    pub fn place_pile_on_bottom(&mut self, mut pile: Pile) {
        pile.cards.append(&mut self.cards);
        self.cards = pile.cards;
    }

    pub fn draw(&mut self) -> Result<Card, String> {
        if self.cards.is_empty() { return Err("Tried to draw card when there was no more cards in the deck.".to_string()); }
        Ok(self.cards.pop().unwrap())
    }

    pub fn size(&self) -> usize {
        self.cards.len()
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
        // TODO: This is probably a bad way of doing this. Also, I don't really
        // understand the code, I just copied it from StackOverflow.
        //
        // We can't do this in the obvious way, [[Pile::new(); 5]; 5], because
        // Pile does not (and cannot) implement Copy.
        //
        // https://stackoverflow.com/questions/31360993/what-is-the-proper-way-to-initialize-a-fixed-length-array
        let stacks = unsafe {
            let mut result: [[Pile; 5]; 5] = std::mem::uninitialized();
            for (_, element) in result.iter_mut().enumerate() {
                let mut value: [Pile; 5] = std::mem::uninitialized();
                for (_, element) in value.iter_mut().enumerate() {
                    std::ptr::write(element, Pile::new());
                }
                std::ptr::write(element, value);
            }
            result
        };

        Board { stacks }
    }

    pub fn get_pile_at(&self, pos: BoardPosition) -> &Pile {
        &self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize]
    }


    pub fn get_card_at(&self, pos: BoardPosition) -> Option<Card> {
        self.get_pile_at(pos).top()
    }

    pub fn place_card_at(&mut self, pos: BoardPosition, card: Card) {
        self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize].place_card_on_top(card);
    }

    pub fn remove_pile_at(&mut self, pos: BoardPosition) {
        self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize] = Pile::new();
    }

    pub fn take_pile_at(&mut self, pos: BoardPosition) -> Pile {
        std::mem::replace(&mut self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize], Pile::new())
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
                    royals_pile.place_card_on_top(card);
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

    // TODO: remove_pile_at, move_pile_to_bottom_of_deck_at: Game itself should
    // know when to move piles around.

    pub fn remove_pile_at(&mut self, pos: BoardPosition) {
        self.board.remove_pile_at(pos);
    }

    pub fn move_pile_to_bottom_of_deck_at(&mut self, pos: BoardPosition) {
        self.deck.place_pile_on_bottom(self.board.take_pile_at(pos));
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

    pub fn cards_left(&self) -> usize {
        self.deck.size()
    }
}

