mod game;
mod geometry;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::mouse::{MouseButton, Cursor, SystemCursor};
use sdl2::ttf::{Font};

use game::{BoardPosition, Game, Card, Suit};
use geometry::{align_text, HorizontalAlignment as AlignH, VerticalAlignment as AlignV};

// TODO: We do integer conversions (mostly between i32 and u32) in a lot of
// places. Is there a way to avoid that? Should we switch to safe conversions?

const CARD_WIDTH: u32 = 150;
const CARD_SPACE: i32 = 25;
const CARD_FONT_HEIGHT: u16 = 40;
const CARD_TEXT_MARGIN: i32 = 20;
const ROYAL_BORDER_WIDTH: u32 = 10;
const UI_FONT_HEIGHT: u16 = 25;
const UI_SPACE: i32 = 8;
const DRAW_PILE_POSITION: (i32, i32) = ((CARD_SPACE + CARD_WIDTH as i32) * 5 + CARD_SPACE, CARD_SPACE);

fn translate_screen_to_board((x, y): (i32, i32)) -> Option<BoardPosition> {
    let board_x = x / (CARD_SPACE + CARD_WIDTH as i32) - 2;
    let board_y = y / (CARD_SPACE + CARD_WIDTH as i32) - 2;
    BoardPosition::new((board_x as i8, board_y as i8)).ok()
}

fn translate_board_to_screen(pos: BoardPosition) -> (i32, i32) {
    let screen_x = CARD_SPACE + (CARD_SPACE + CARD_WIDTH as i32) * (pos.x() as i32 + 2);
    let screen_y = CARD_SPACE + (CARD_SPACE + CARD_WIDTH as i32) * (pos.y() as i32 + 2);
    (screen_x, screen_y)
}

struct DrawContext<'a> {
    pub canvas: &'a mut WindowCanvas,
    pub texture_creator: &'a TextureCreator<WindowContext>,
    pub ui_font: &'a Font<'a, 'static>,
    pub card_font: &'a Font<'a, 'static>,
}

fn draw_text<'a>(context: &mut DrawContext, font: &'a Font<'a, 'static>, text: &'a str, color: Color, (x, y): (i32, i32)) {
    let (w, h) = font.size_of(text).unwrap();
    let text_surface = font.render(text).blended(color).unwrap();
    let text_texture = context.texture_creator.create_texture_from_surface(text_surface).unwrap();
    context.canvas.copy(&text_texture, None, Rect::new(x, y, w, h)).unwrap();
}

fn draw_text_align<'a>(context: &mut DrawContext, font: &'a Font<'a, 'static>, text: &'a str, color: Color,
                       parent: Rect, horizontal_alignment: AlignH, vertical_alignment: AlignV, horizontal_margin: i32, vertical_margin: i32) {
    let pos = align_text(font, text, parent, horizontal_alignment, vertical_alignment, horizontal_margin, vertical_margin);
    draw_text(context, font, text, color, pos);
}

fn color_for_suit(suit: Suit, is_active: bool) -> Color {
    use Suit::*;
    if is_active {
        match suit {
            Hearts => { Color::RGB(0xD0, 0x60, 0x60) },
            Diamonds => { Color::RGB(0xC8, 0x78, 0x60) },
            Spades => { Color::RGB(0x60, 0x60, 0xD0) },
            Clubs => { Color::RGB(0x60, 0x90, 0xB8) },
            Joker => { Color::RGB(0x40, 0x40, 0x40) },
        }
    } else {
        match suit {
            Hearts => { Color::RGB(0xD0, 0xB0, 0xB0) },
            Diamonds => { Color::RGB(0xC8, 0xB2, 0xAA) },
            Spades => { Color::RGB(0xB0, 0xB0, 0xD0) },
            Clubs => { Color::RGB(0xA0, 0xB2, 0xC8) },
            Joker => { Color::RGB(0xA0, 0xA0, 0xA0) },
        }
    }
}

fn draw_card(context: &mut DrawContext, card: Option<Card>, (x, y): (i32, i32)) {
    let rect = Rect::new(x, y, CARD_WIDTH, CARD_WIDTH);
    match card {
        Some(card) => {
            let color = color_for_suit(card.suit(), true);
            if card.is_royal() {
                // Draw border
                context.canvas.set_draw_color(Color::RGB(0xE0, 0xA8, 0x38));
                context.canvas.fill_rect(rect).unwrap();
                // Draw card
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(Rect::new(
                    x + ROYAL_BORDER_WIDTH as i32,
                    y + ROYAL_BORDER_WIDTH as i32,
                    CARD_WIDTH - (ROYAL_BORDER_WIDTH * 2),
                    CARD_WIDTH - (ROYAL_BORDER_WIDTH * 2)
                )).unwrap();
            } else {
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(rect).unwrap();
            }
            draw_text_align(context, context.card_font, &format!("{}", card.value()), Color::RGB(0xFF, 0xFF, 0xFF),
                            rect, AlignH::Left, AlignV::Top, CARD_TEXT_MARGIN, CARD_TEXT_MARGIN);
        },
        None => {
            context.canvas.set_draw_color(Color::RGB(0xE2, 0xDB, 0xD8));
            context.canvas.fill_rect(rect).unwrap();
        },
    }
}

fn draw_inactive_card(context: &mut DrawContext, card: Option<Card>, (x, y): (i32, i32)) {
    let rect = Rect::new(x, y, CARD_WIDTH, CARD_WIDTH);
    match card {
        Some(card) => {
            let color = color_for_suit(card.suit(), false);
            if card.is_royal() {
                // Draw border
                context.canvas.set_draw_color(Color::RGB(0xE0, 0xC8, 0xB8));
                context.canvas.fill_rect(rect).unwrap();
                // Draw card
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(Rect::new(
                    x + ROYAL_BORDER_WIDTH as i32,
                    y + ROYAL_BORDER_WIDTH as i32,
                    CARD_WIDTH - (ROYAL_BORDER_WIDTH * 2),
                    CARD_WIDTH - (ROYAL_BORDER_WIDTH * 2)
                )).unwrap();
            } else {
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(rect).unwrap();
            }
            draw_text_align(context, context.card_font, &format!("{}", card.value()), Color::RGB(0xFF, 0xFF, 0xFF),
                            rect, AlignH::Left, AlignV::Top, CARD_TEXT_MARGIN, CARD_TEXT_MARGIN);
        },
        None => {
            context.canvas.set_draw_color(Color::RGB(0xED, 0xE8, 0xE4));
            context.canvas.fill_rect(rect).unwrap();
        },
    }
}

fn draw_armor(context: &mut DrawContext, armor: u8, (x, y): (i32, i32)) {
    if armor > 0 {
        let rect = Rect::new(x, y, CARD_WIDTH, CARD_WIDTH);
        draw_text_align(context, context.ui_font, &format!("+{}", armor), Color::RGB(0xFF, 0xFF, 0xFF),
                        rect, AlignH::Left, AlignV::Bottom, CARD_TEXT_MARGIN, CARD_TEXT_MARGIN);
    }
}

fn draw_card_on_board(context: &mut DrawContext, game: &Game, pos: BoardPosition, draw_placability: bool) {
    let card = game.get_card_at(pos);
    let (x, y) = translate_board_to_screen(pos);
    if draw_placability && !game.can_place_at(pos) {
        draw_inactive_card(context, card, (x, y));
    } else {
        draw_card(context, card, (x, y));
    }
    draw_armor(context, game.get_armor_at(pos), (x, y));
}

pub fn main() {
    let mut game = Game::new();
    game.set_up();

    let mut dragged_card: Option<Card> = None;
    let mut dragged_offset: Option<(i32, i32)> = None;

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let ttf = sdl2::ttf::init().unwrap();

    let ui_font = ttf.load_font("./sansb.ttf", UI_FONT_HEIGHT).unwrap();
    let card_font = ttf.load_font("./sansb.ttf", CARD_FONT_HEIGHT).unwrap();

    let window = video.window("Squaretillery", (CARD_WIDTH + CARD_SPACE as u32) * 6 + CARD_SPACE as u32, (CARD_WIDTH + CARD_SPACE as u32) * 5 + CARD_SPACE as u32)
        .position_centered()
        //.allow_highdpi()
        .build().unwrap();

    let mut canvas = window.into_canvas()
        //.present_vsync()
        .build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().unwrap();

    let mut context = DrawContext {
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        ui_font: &ui_font,
        card_font: &card_font,
    };

    let cursor_default = Cursor::from_system(SystemCursor::Arrow).unwrap();
    let cursor_hand = Cursor::from_system(SystemCursor::Hand).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    if game.drawn().is_some() {
                        game.add_to_shame_pile();
                    }
                },
                Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                    if x >= DRAW_PILE_POSITION.0 &&
                        x <= DRAW_PILE_POSITION.0 + CARD_WIDTH as i32 &&
                        y >= DRAW_PILE_POSITION.1 &&
                        y <= DRAW_PILE_POSITION.1 + CARD_WIDTH as i32
                    {
                        if let Some(card) = game.drawn() {
                            dragged_card = Some(card);
                            dragged_offset = Some((x - DRAW_PILE_POSITION.0, y - DRAW_PILE_POSITION.1));
                        } else {
                            game.draw().unwrap();
                        }
                    }
                },
                Event::MouseButtonUp { x, y, mouse_btn: MouseButton::Left, .. } => {
                    if dragged_card.is_some() {
                        translate_screen_to_board((x, y)).map(|pos| {
                            game.place_card_at(pos).unwrap();
                        });

                        dragged_card = None;
                        dragged_offset = None;
                    }
                },
                Event::MouseButtonUp { x, y, mouse_btn: MouseButton::Middle, .. } => {
                    translate_screen_to_board((x, y)).map(|pos| {
                        game.remove_pile_at(pos)
                    });
                },
                Event::MouseButtonUp { x, y, mouse_btn: MouseButton::Right, .. } => {
                    translate_screen_to_board((x, y)).map(|pos| {
                        game.move_pile_to_bottom_of_deck_at(pos)
                    });
                },
                _ => {}
            }
        }


        context.canvas.set_draw_color(Color::RGB(0xF2, 0xEB, 0xE8));
        context.canvas.clear();

        // Change mouse cursor
        let (mouse_x, mouse_y) = {
            let mouse_state = event_pump.mouse_state();
            (mouse_state.x(), mouse_state.y())
        };

        if mouse_x >= DRAW_PILE_POSITION.0 &&
            mouse_x <= DRAW_PILE_POSITION.0 + CARD_WIDTH as i32 &&
            mouse_y >= DRAW_PILE_POSITION.1 &&
            mouse_y <= DRAW_PILE_POSITION.1 + CARD_WIDTH as i32 &&
            game.drawn().is_none()
        {
            // Cursor is on draw pile and there is no card drawn.
            cursor_hand.set();
        } else {
            cursor_default.set();
        }

        // Render current (drawn) card

        if dragged_card.is_none() {
            draw_card(&mut context, game.drawn(), DRAW_PILE_POSITION);
        } else {
            draw_card(&mut context, None, DRAW_PILE_POSITION);
        }

        (|context: &mut DrawContext| {
            draw_text_align(context, context.ui_font, &format!("{} LEFT", game.cards_left()), Color::RGB(0x82, 0x7B, 0x78),
                            Rect::new(DRAW_PILE_POSITION.0, DRAW_PILE_POSITION.1 + CARD_WIDTH as i32, CARD_WIDTH, 0),
                            AlignH::Center, AlignV::Top, 0, UI_SPACE);
        })(&mut context);

        // Render shame

        if game.get_shame() > 0 {
            (|context: &mut DrawContext| {
                draw_text_align(context, context.ui_font, &format!("{} SHAME", game.get_shame()), Color::RGB(0xC2, 0x7B, 0x78),
                                Rect::new(DRAW_PILE_POSITION.0, DRAW_PILE_POSITION.1 + CARD_WIDTH as i32 + UI_SPACE + UI_FONT_HEIGHT as i32, CARD_WIDTH, 0),
                                AlignH::Center, AlignV::Top, 0, UI_SPACE);
            })(&mut context);
        }

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if let Ok(pos) = BoardPosition::new((x, y)) {
                    draw_card_on_board(&mut context, &game, pos, dragged_card.is_some());
                }
            }
        }

        // Render card being dragged

        dragged_card.map(|card| {
            draw_card(&mut context, Some(card), (event_pump.mouse_state().x() - dragged_offset.unwrap().0, event_pump.mouse_state().y() - dragged_offset.unwrap().1));
        });

        context.canvas.present();
    }
}
