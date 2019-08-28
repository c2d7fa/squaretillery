use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::mouse::MouseButton;
use sdl2::ttf::{Font};

mod game;

fn translate_screen_to_board((x, y): (i32, i32)) -> Option<(i8, i8)> {
    let width = 150;
    let margin = 25;

    let board_x = x / (margin + width) - 2;
    let board_y = y / (margin + width) - 2;

    if (board_x == 2 || board_x == -2) && (board_y == 2 || board_y == -2) {
        None
    } else if board_x > 2 || board_y > 2 {
        None
    } else {
        Some((board_x as i8, board_y as i8))
    }
}

struct DrawContext<'a> {
    pub canvas: &'a mut WindowCanvas,
    pub texture_creator: &'a TextureCreator<WindowContext>,
    pub font: &'a Font<'a, 'static>,
}

fn draw_text<'b>(context: &mut DrawContext, text: &'b str, color: Color, (x, y): (i32, i32)) {
    let (w, h) = context.font.size_of(text).unwrap();
    let text_surface = context.font.render(text).blended(color).unwrap();
    let text_texture = context.texture_creator.create_texture_from_surface(text_surface).unwrap();
    context.canvas.copy(&text_texture, None, Rect::new(x, y, w, h)).unwrap();
}

fn draw_card(context: &mut DrawContext, card: game::Card, (x, y): (i32, i32)) {
    context.canvas.set_draw_color(Color::RGB(0xC8, 0xC8, 0xC8));
    context.canvas.fill_rect(Rect::new(x, y, 150, 150)).unwrap();
    draw_text(context, &format!("{}", card.format_text_short()), Color::RGB(0x00, 0x00, 0x00), (x + 5, y + 5));
}

fn draw_card_on_board(context: &mut DrawContext, card: Option<game::Card>, (x, y): (i8, i8)) {
    fn rect_for_card((x, y): (i8, i8)) -> Rect {
        let (x_, y_) = (i32::from(x) + 2, i32::from(y) + 2);
        return Rect::new(25 + (25 + 150) * x_, 25 + (25 + 150) * y_, 150, 150);
    }

    match card {
        Some(card) => {
            context.canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            let rect = rect_for_card((x, y));
            context.canvas.fill_rect(rect).unwrap();
            draw_text(context, &format!("{}", card.format_text_short()), Color::RGB(0x00, 0x00, 0x00), (rect.x() + 5, rect.y() + 5));
        },
        None => {
            let rect = rect_for_card((x, y));
            context.canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            context.canvas.draw_rect(rect).unwrap();
            draw_text(context, "No card", Color::RGB(0xFF, 0xFF, 0xFF), (rect.x() + 5, rect.y() + 5));
        },
    }
}

pub fn main() {
    let mut game = game::Game::new();

    let mut dragged_card: Option<game::Card> = None;
    let mut dragged_offset: Option<(i32, i32)> = None;

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let ttf = sdl2::ttf::init().unwrap();

    let font = ttf.load_font("./sans.ttf", 22).unwrap();

    let window = video.window("Gridcannon", 925 + 25 + 200 + 25, 925)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().unwrap();

    let mut context = DrawContext {
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        font: &font,
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
                        translate_screen_to_board((x, y)).map(|(board_x, board_y)| {
                            game.place_card_at(board_x, board_y).unwrap();
                        });

                        dragged_card = None;
                        dragged_offset = None;
                    }
                },
                Event::MouseButtonUp { x, y, mouse_btn: MouseButton::Middle, .. } => {
                    translate_screen_to_board((x, y)).map(|(board_x, board_y)| {
                        game.remove_card_at(board_x, board_y).unwrap()
                    });
                },
                _ => {}
            }
        }

        context.canvas.set_draw_color(Color::RGB(0x60, 0x60, 0x60));
        context.canvas.clear();



        // Render current (drawn) card

        if dragged_card.is_none() {
            draw_card_on_board(&mut context, game.drawn(), (3, -2));
        }

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if !((x == -2 || x == 2) && (y == -2 || y == 2)) {
                    draw_card_on_board(&mut context, game.get_card_at(x, y).unwrap(), (x, y));
                }
            }
        }

        // Render card being dragged

        dragged_card.map(|card| {
            draw_card(&mut context, card, (event_pump.mouse_state().x() - dragged_offset.unwrap().0, event_pump.mouse_state().y() - dragged_offset.unwrap().1));
        });

        context.canvas.present();
    }
}
