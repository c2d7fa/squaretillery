use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::mouse::MouseButton;
use sdl2::ttf::{Font};

mod game;
use game::{BoardPosition};

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
}

// TODO: Clean up
fn draw_card_text<'b>(context: &mut DrawContext, text: &'b str, color: Color, (x, y): (i32, i32)) {
    let (w, h) = context.card_font.size_of(text).unwrap();
    let text_surface = context.card_font.render(text).blended(color).unwrap();
    let text_texture = context.texture_creator.create_texture_from_surface(text_surface).unwrap();
    context.canvas.copy(&text_texture, None, Rect::new(x, y, w, h)).unwrap();
}

fn color_for_suit(suit: game::Suit) -> Color {
    match suit {
        game::Suit::Hearts => { Color::RGB(0xD0, 0x60, 0x60) },
        game::Suit::Diamonds => { Color::RGB(0xC8, 0x78, 0x60) },
        game::Suit::Spades => { Color::RGB(0x60, 0x60, 0xD0) },
        game::Suit::Clubs => { Color::RGB(0x60, 0x90, 0xB8) },
    }
}

fn draw_card(context: &mut DrawContext, card: Option<game::Card>, (x, y): (i32, i32)) {
    let rect = Rect::new(x, y, 150, 150);
    match card {
        Some(card) => {
            let color = color_for_suit(card.suit());
            if card.is_royal() {
                context.canvas.set_draw_color(Color::RGB(0xE0, 0xA8, 0x38));
                context.canvas.fill_rect(rect).unwrap();
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(Rect::new(x + 8, y + 8, 150 - 16, 150 - 16));
            } else {
                context.canvas.set_draw_color(color);
                context.canvas.fill_rect(rect).unwrap();
            }
            draw_card_text(context, &format!("{}", card.value()), Color::RGB(0xFF, 0xFF, 0xFF), (rect.x() + 20, rect.y() + 20));
        },
        None => {
            context.canvas.set_draw_color(Color::RGB(0xF2, 0xEB, 0xE8));
            context.canvas.set_draw_color(Color::RGB(0xE2, 0xDB, 0xD8));
            context.canvas.fill_rect(rect).unwrap();
        },
    }
}

fn draw_card_on_board(context: &mut DrawContext, card: Option<game::Card>, pos: BoardPosition) {
    draw_card(context, card, (25 + (25 + 150) * (pos.x() as i32 + 2), 25 + (25 + 150) * (pos.y() as i32 + 2)));
}

pub fn main() {
    let mut game = game::Game::new();
    game.set_up();

    let mut dragged_card: Option<game::Card> = None;
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

        draw_text(&mut context, &format!("{} LEFT", game.cards_left()), Color::RGB(0x82, 0x7B, 0x78), ((25 + 150) * 5 + 25 + 25, 25 + 150 + 8));

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if let Ok(pos) = BoardPosition::new((x, y)) {
                    draw_card_on_board(&mut context, game.get_card_at(pos), pos);
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
