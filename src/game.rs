#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
    Joker,
}

impl Suit {
    fn is_same_color_as(self, other: Suit) -> bool {
        use Suit::*;
        if self == Spades || self == Clubs { other == Spades || other == Clubs }
        else if self == Hearts || self == Diamonds { other == Hearts || other == Diamonds }
        else { false }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Card {
    suit: Suit,
    value: u8,  // NOTE: Not used when suit is Joker.
}

#[derive(Debug)]
pub struct Pile {
    cards: Vec<Card>,  // NOTE: The top of the pile is at the back.
}

#[derive(Debug)]
pub struct Board {
    stacks: [[Pile; 5]; 5],
    armor: [[u8; 5]; 5],     // TODO: This is an inelegant representation. Should this even be here, or should it be somewhere else?
}

#[derive(Debug)]
pub struct Game {
    drawn: Option<Card>,  // Last drawn card, currently waiting to be placed
    deck: Pile,           // Remaining cards not on the board
    board: Board,
    shame: u8,
}

impl Card {
    pub fn new(value: u8, suit: Suit) -> Result<Card, String> {
        if value == 0 && suit != Suit::Joker { return Err("Only jokers can have value 0.".to_string()); }
        if value != 0 && suit == Suit::Joker { return Err("Jokers must have value 0.".to_string()); }
        if value > 13 { return Err("Card value cannot be greater than 13.".to_string()); }

        Ok(Card { suit, value })
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
        cards.push(Card::new(0, Suit::Joker).unwrap());
        cards.push(Card::new(0, Suit::Joker).unwrap());

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    pub fn all_valid() -> Vec<BoardPosition> {
        let mut result = vec![];
        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if let Ok(pos) = Self::new((x, y)) {
                    result.push(pos);
                }
            }
        }
        result
    }

    // Return true if self is part of the middle 3x3 grid, false otherwise.
    pub fn is_cannon(&self) -> bool {
        (self.x >= -1 && self.x <= 1) && (self.y >= -1 && self.y <= 1)
    }

    // Return true if self is part of the outer edge of the cannon; that is, if
    // it is not the center card and also is not a royal's place.
    pub fn is_outer_cannon(&self) -> bool {
        self.is_cannon() && !(self.x == 0 && self.y == 0)
    }

    // Return true if self is part of the outer edge that contains the royals,
    // false otherwise.
    pub fn is_edge(&self) -> bool {
        !self.is_cannon()
    }

    pub fn adjacent_edges(&self) -> Vec<BoardPosition> {
        let mut result = vec![];
        BoardPosition::new((self.x - 1, self.y)).map(|pos| { if pos.is_edge() { result.push(pos) } });
        BoardPosition::new((self.x + 1, self.y)).map(|pos| { if pos.is_edge() { result.push(pos) } });
        BoardPosition::new((self.x, self.y - 1)).map(|pos| { if pos.is_edge() { result.push(pos) } });
        BoardPosition::new((self.x, self.y + 1)).map(|pos| { if pos.is_edge() { result.push(pos) } });
        result
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

        Board { stacks, armor: [[0; 5]; 5] }
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
        self.armor[(2 + pos.x()) as usize][(2 + pos.y()) as usize] = 0;
    }

    pub fn take_pile_at(&mut self, pos: BoardPosition) -> Pile {
        self.armor[(2 + pos.x()) as usize][(2 + pos.y()) as usize] = 0;
        std::mem::replace(&mut self.stacks[(2 + pos.x()) as usize][(2 + pos.y()) as usize], Pile::new())
    }

    // TODO: This should probably check whether it even makes sense for that
    // card to have armor.
    pub fn add_armor_at(&mut self, pos: BoardPosition, amount: u8) {
        self.armor[(2 + pos.x()) as usize][(2 + pos.y()) as usize] += amount;
    }

    pub fn get_armor_at(&self, pos: BoardPosition) -> u8 {
        self.armor[(2 + pos.x()) as usize][(2 + pos.y()) as usize]
    }

    fn adjacent_empty_edges(&self, pos: BoardPosition) -> Vec<BoardPosition> {
        let mut result = vec![];
        for adj in pos.adjacent_edges() {
            if self.get_card_at(adj).is_none() { result.push(adj) }
        }
        result
    }

    // TODO: Error checking
    pub fn find_valid_royal_placement_positions(&self, royal: Card) -> Vec<BoardPosition> {
        let mut result = vec![];

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        enum SuitSimilarity { None, Color, Suit };

        let mut best_suit_similarity = SuitSimilarity::None;
        let mut best_value = 0;

        for pos in BoardPosition::all_valid() {
            if !pos.is_outer_cannon() { continue }

            if self.adjacent_empty_edges(pos).is_empty() { continue }

            let suit_similarity = {
                if self.get_card_at(pos).unwrap().suit() == royal.suit() { SuitSimilarity::Suit }
                else if self.get_card_at(pos).unwrap().suit().is_same_color_as(royal.suit()) { SuitSimilarity::Color }
                else { SuitSimilarity::None }
            };
            let value = self.get_card_at(pos).unwrap().value();

            if suit_similarity < best_suit_similarity { continue }
            else if suit_similarity == best_suit_similarity { /* Do nothing */ }
            else {
                best_suit_similarity = suit_similarity;
                result.clear();
                best_value = value;
            }

            if value < best_value { continue }
            else if value == best_value { /* Do nothing */ }
            else {
                result.clear();
                best_value = value;
            }

            for adj in self.adjacent_empty_edges(pos) { result.push(adj) }
        }

        result
    }
}

impl Game {
    pub fn new() -> Game {
        Game { drawn: None, deck: Pile::new_shuffled_deck(), board: Board::new_empty(), shame: 0 }
    }

    pub fn set_up(&mut self) {
        let mut royals_pile = Pile::new();

        for position in BoardPosition::all_valid() {
            if !position.is_outer_cannon() { continue }
            'search_card: loop {
                let card = self.deck.draw().unwrap();
                if card.is_royal() {
                    royals_pile.place_card_on_top(card);
                } else {
                    self.board.place_card_at(position, card);
                    break 'search_card;
                }
            }
        }

        self.deck.place_pile_on_top(royals_pile);
    }

    pub fn can_place_at(&self, pos: BoardPosition) -> bool {
        // TODO: We should probably clean up this code a litte bit...
        if self.drawn.is_none() {
            false
        } else {
            let drawn = self.drawn.unwrap();
            if drawn.is_royal() {
                self.board.find_valid_royal_placement_positions(drawn).contains(&pos)
            } else {
                if pos.is_cannon() {
                    match self.get_card_at(pos) {
                        None => true,
                        Some(card) => {
                            drawn.value() == 1 || drawn.value() >= card.value()
                        },
                    }
                } else {
                    false
                }
            }
        }
    }

    pub fn get_card_at(&self, pos: BoardPosition) -> Option<Card> {
        self.board.get_card_at(pos)
    }

    pub fn place_card_at(&mut self, pos: BoardPosition) -> Result<(), String> {
        if self.drawn.is_none() {
            Err("Cannot place card because there is no drawn card.".to_string())
        } else {
            let card = self.get_card_at(pos);
            if card.is_some() && card.unwrap().is_royal() {
                self.add_armor_at(pos).unwrap();
            } else if self.drawn.unwrap().value() == 1 || self.drawn.unwrap().value() == 0 {
                self.move_pile_to_bottom_of_deck_at(pos);
                self.board.place_card_at(pos, self.drawn.unwrap());
                self.drawn = None;
            } else {
                self.board.place_card_at(pos, self.drawn.unwrap());
                self.drawn = None;
            }
            Ok(())
        }
    }

    pub fn add_armor_at(&mut self, pos: BoardPosition) -> Result<(), String> {
        if self.drawn.is_none() {
            Err("Cannot add armor because no card is drawn.".to_string())
        } else {
            self.board.add_armor_at(pos, self.drawn.unwrap().value());
            self.drawn = None;
            Ok(())
        }
    }

    pub fn get_armor_at(&self, pos: BoardPosition) -> u8 {
        self.board.get_armor_at(pos)
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

    pub fn add_to_shame_pile(&mut self) {
        self.drawn = None;
        self.shame += 1;
    }

    pub fn get_shame(&self) -> u8 {
        self.shame
    }
}

