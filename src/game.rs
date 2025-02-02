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

    pub fn royals_left(&self) -> usize {
        self.cards.iter().filter(|c| { c.is_royal() }).count()
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
        // TODO: Clean this up
        let mut result = vec![];
        BoardPosition::new((self.x - 1, self.y)).map(|pos| { if pos.is_edge() { result.push(pos) } }).unwrap_or(());
        BoardPosition::new((self.x + 1, self.y)).map(|pos| { if pos.is_edge() { result.push(pos) } }).unwrap_or(());
        BoardPosition::new((self.x, self.y - 1)).map(|pos| { if pos.is_edge() { result.push(pos) } }).unwrap_or(());
        BoardPosition::new((self.x, self.y + 1)).map(|pos| { if pos.is_edge() { result.push(pos) } }).unwrap_or(());
        result
    }

    // If self is outer cannon, return the positions that are "aimed at", that
    // is, the royals that could be killed by placing a card here. In other
    // cases, return an empty vector.
    fn aimed_at(&self) -> Vec<BoardPosition> {
        // TODO: This is not the most elegant algorithm. Not sure if there is a
        // better way? Or maybe we should just hardcode all of the positions?
        if (self.x, self.y) == (-1, -1) {
            vec![BoardPosition::new((-1, 2)).unwrap(), BoardPosition::new((2, -1)).unwrap()]
        } else if (self.x, self.y) == (-1, 0) {
            vec![BoardPosition::new((2, 0)).unwrap()]
        } else if (self.x, self.y) == (0, -1) {
            vec![BoardPosition::new((0, 2)).unwrap()]
        } else if self.x > 0 {
            BoardPosition::new((-self.x, self.y)).unwrap().aimed_at().into_iter().map(|p| { BoardPosition::new((-p.x(), p.y())).unwrap() }).collect()
        } else if self.y > 0 {
            BoardPosition::new((self.x, -self.y)).unwrap().aimed_at().into_iter().map(|p| { BoardPosition::new((p.x(), -p.y())).unwrap() }).collect()
        } else {
            vec![]
        }
    }

    // If self is the position of a royal, return the tiles that make up the
    // "cannon", that is, the tiles whose sum would be used as the damage when
    // attacking the royal at self. In other cases, return an empty vector.
    fn cannon_towards(&self) -> Vec<BoardPosition> {
        if self.x == -2 { vec![BoardPosition::new((-1, self.y)).unwrap(), BoardPosition::new((0, self.y)).unwrap()] }
        else if self.x == 2 { vec![BoardPosition::new((0, self.y)).unwrap(), BoardPosition::new((1, self.y)).unwrap()] }
        else if self.y == -2 { vec![BoardPosition::new((self.x, -1)).unwrap(), BoardPosition::new((self.x, 0)).unwrap()] }
        else if self.y == 2 { vec![BoardPosition::new((self.x, 0)).unwrap(), BoardPosition::new((self.x, 1)).unwrap()] }
        else { vec![] }
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

    pub fn resolve_attack(&mut self, royal: BoardPosition) {
        if let Some(royal_card) = self.get_card_at(royal) {
            let health = royal_card.value() + self.get_armor_at(royal);

            let mut is_valid_attack = true;

            let mut damage = 0;
            for cannon_pos in royal.cannon_towards() {
                if let Some(card) = self.get_card_at(cannon_pos) {
                    if royal_card.value() == 12 && !card.suit().is_same_color_as(royal_card.suit()) { is_valid_attack = false; }
                    if royal_card.value() == 13 && card.suit() != royal_card.suit() { is_valid_attack = false; }
                    damage += card.value();
                } else {
                    is_valid_attack = false;
                }
            }

            if damage >= health && is_valid_attack {
                self.remove_pile_at(royal);
            }
        }
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
                    if self.get_card_at(pos).is_some() {
                        true  // Can place card to add armor
                    } else {
                        false
                    }
                }
            }
        }
    }

    pub fn get_card_at(&self, pos: BoardPosition) -> Option<Card> {
        self.board.get_card_at(pos)
    }

    pub fn place_card_at(&mut self, pos: BoardPosition) -> Option<()> {
        if !self.can_place_at(pos) { return None; }

        if self.drawn.is_none() {
            panic!();
        } else {
            let card = self.get_card_at(pos);
            if card.is_some() && card.unwrap().is_royal() {
                self.add_armor_at(pos).unwrap();
            } else {
                if self.drawn.unwrap().value() == 1 || self.drawn.unwrap().value() == 0 {
                    self.move_pile_to_bottom_of_deck_at(pos);
                }

                for attacked in pos.aimed_at() {
                    self.board.resolve_attack(attacked);
                }

                self.board.place_card_at(pos, self.drawn.unwrap());
                self.drawn = None;
            }
            Some(())
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

    fn move_pile_to_bottom_of_deck_at(&mut self, pos: BoardPosition) {
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

    pub fn is_game_over(&self) -> bool {
        self.deck.royals_left() == 0 ||
            self.deck.size() == 0  // No possible actions (TODO: Handle this case in scoring)
    }
}

