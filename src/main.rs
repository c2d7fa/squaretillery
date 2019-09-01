mod game;
mod geometry;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::mouse::MouseButton;
use sdl2::ttf::{Font};

use game::{BoardPosition, Game, Card, Suit};
use geometry::{align_text, HorizontalAlignment as AlignH, VerticalAlignment as AlignV};

fn translate_screen_to_board((x, y): (i32, i32)) -> Option<BoardPosition> {
    let width = 150;
    let margin = 25;

    let board_x = x / (margin + width) - 2;
    let board_y = y / (margin + width) - 2;

    BoardPosition::new((board_x as i8, board_y as i8)).ok()  // TODO: Use safe conversion
}

struct DrawContext<'a> {
    pub canvas: &'a mut WindowCanvas,
    pub texture_creator: &'a TextureCreator<WindowContext>,
    pub font: &'a Font<'a, 'static>,
    pub card_font: &'a Font<'a, 'static>,
}

fn draw_text<'b>(context: &mut DrawContext, text: &'b str, color: Color, (x, y): (i32, i32)) {
    let (w, h) = context.font.size_of(text).unwrap();
    let text_surface = context.font.render(text).blended(color).unwrap();
    let text_texture = context.texture_creator.create_texture_from_surface(text_surface).unwrap();
    context.canvas.copy(&text_texture, None, Rect::new(x, y, w, h)).unwrap();
//    context.canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
//    context.canvas.draw_rect(Rect::new(x, y, w, h)).unwrap();
}

// TODO: Clean up
fn draw_card_text<'b>(context: &mut DrawContext, text: &'b str, color: Color, (x, y): (i32, i32)) {
    let (w, h) = context.card_font.size_of(text).unwrap();
    let text_surface = context.card_font.render(text).blended(color).unwrap();
    let text_texture = context.texture_creator.create_texture_from_surface(text_surface).unwrap();
    context.canvas.copy(&text_texture, None, Rect::new(x, y, w, h)).unwrap();
//    context.canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
//    context.canvas.draw_rect(Rect::new(x, y, w, h)).unwrap();
}

fn color_for_suit(suit: Suit) -> Color {
    use Suit::*;
    match suit {
        Hearts => { Color::RGB(0xD0, 0x60, 0x60) },
        Diamonds => { Color::RGB(0xC8, 0x78, 0x60) },
        Spades => { Color::RGB(0x60, 0x60, 0xD0) },
        Clubs => { Color::RGB(0x60, 0x90, 0xB8) },
    }
}

fn draw_card(context: &mut DrawContext, card: Option<Card>, (x, y): (i32, i32)) {
    let rect = Rect::new(x, y, 150, 150);
    match card {
        Some(card) => {
            let color = color_for_suit(card.suit());
            if card.is_royal() {
                context.canvas.set_draw_color(Color::RGB(0xE0, 0xA8, 0x38));
                context.canvas.fill_rect(rect).unwrap();
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(Rect::new(x + 8, y + 8, 150 - 16, 150 - 16)).unwrap();
            } else {
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(rect).unwrap();
            }
            draw_card_text(context, &format!("{}", card.value()), Color::RGB(0xFF, 0xFF, 0xFF), align_text(context.card_font, &format!("{}", card.value()), rect, AlignH::Left, AlignV::Top, 20, 20));
        },
        None => {
            context.canvas.set_draw_color(Color::RGB(0xF2, 0xEB, 0xE8));
            context.canvas.set_draw_color(Color::RGB(0xE2, 0xDB, 0xD8));
            context.canvas.fill_rect(rect).unwrap();
        },
    }
}

fn draw_armor(context: &mut DrawContext, armor: u8, (x, y): (i32, i32)) {
    if armor > 0 {
        let rect = Rect::new(x, y, 150, 150);
        draw_text(context, &format!("+{}", armor), Color::RGB(0xFF, 0xFF, 0xFF), align_text(context.font, &format!("+{}", armor), rect, AlignH::Left, AlignV::Bottom, 20, 20));
    }
}

fn draw_card_on_board(context: &mut DrawContext, game: &Game, pos: BoardPosition) {
    let card = game.get_card_at(pos);
    let (x, y) = (25 + (25 + 150) * (pos.x() as i32 + 2), 25 + (25 + 150) * (pos.y() as i32 + 2));
    draw_card(context, card, (x, y));
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

    let font = ttf.load_font("./sansb.ttf", 22).unwrap();
    let card_font = ttf.load_font("./sansb.ttf", 38).unwrap();

    let window = video.window("Gridcannon", 900 + 150 + 25, 900)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().unwrap();

    let mut context = DrawContext {
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        font: &font,
        card_font: &card_font,
    };

    'running: loop {
        let mouse_state = event_pump.mouse_state();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if game.drawn().is_none() {
                        game.draw().unwrap();
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    if game.drawn().is_some() {
                        translate_screen_to_board((mouse_state.x(), mouse_state.y())).map(|pos| {
                            game.add_armor_at(pos).unwrap();
                        });
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    if game.drawn().is_some() {
                        game.add_to_shame_pile();
                    }
                },
                Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                    if y >= 25 && y <= 150 + 25 && x >= (25 + 150) * 5 + 25 && x <= (25 + 150) * 5 + 25 + 150 {
                        game.drawn().map(|card| {
                            dragged_card = Some(card);
                            dragged_offset = Some((x - ((25 + 150) * 5 + 25), y - 25));
                        });
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

        // Render current (drawn) card

        if dragged_card.is_none() {
            draw_card(&mut context, game.drawn(), ((25 + 150) * 5 + 25, 25));
        } else {
            draw_card(&mut context, None, ((25 + 150) * 5 + 25, 25));
        }

        {
            let pos = align_text(context.font, &format!("{} LEFT", game.cards_left()), Rect::new((25 + 150) * 5 + 25, 25 + 150, 150, 0), AlignH::Center, AlignV::Top, 0, 8);
            draw_text(&mut context, &format!("{} LEFT", game.cards_left()), Color::RGB(0x82, 0x7B, 0x78), pos)
        };

        // Render shame

        if game.get_shame() > 0 {
            {
                let pos = align_text(context.font, &format!("{} SHAME", game.get_shame()), Rect::new((25 + 150) * 5 + 25, 25 + 150 + 8 + 22, 150, 0), AlignH::Center, AlignV::Top, 0, 8);
                draw_text(&mut context, &format!("{} SHAME", game.get_shame()), Color::RGB(0xC2, 0x7B, 0x78), pos)
            };
        }

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if let Ok(pos) = BoardPosition::new((x, y)) {
                    draw_card_on_board(&mut context, &game, pos);
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
