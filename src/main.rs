use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use sdl2::ttf::{Font};

mod game;

fn draw_text<'a, T: RenderTarget, U>(canvas: &mut Canvas<T>, texture_creator: &TextureCreator<U>, text: &'a str, font: &Font, color: Color, coord: (i32, i32))  {
    let text_size = font.size_of(text).unwrap();
    let text_surface = font.render(text).blended(color).unwrap();
    let text_texture = texture_creator.create_texture_from_surface(text_surface).unwrap();
    canvas.copy(&text_texture, None, Rect::new(coord.0, coord.1, text_size.0, text_size.1)).unwrap();
}

fn draw_card<'a, T: RenderTarget, U>(canvas: &mut Canvas<T>, texture_creator: &TextureCreator<U>, font: &Font, card: Option<game::Card>, (x, y): (i8, i8)) {
    match card {
        Some(card) => {
            canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            let rect = rect_for_card((x, y));
            canvas.fill_rect(rect).unwrap();
            draw_text(canvas, texture_creator, &format!("{}", card.format_text_short()), font, Color::RGB(0x00, 0x00, 0x00), (rect.x() + 5, rect.y() + 5));
        },
        None => {
            let rect = rect_for_card((x, y));
            canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            canvas.draw_rect(rect).unwrap();
            draw_text(canvas, texture_creator, "No card", font, Color::RGB(0xFF, 0xFF, 0xFF), (rect.x() + 5, rect.y() + 5));
        },
    }
}

fn rect_for_card((x, y): (i8, i8)) -> Rect {
    let (x_, y_) = (x as i32 + 2, y as i32 + 2);
    return Rect::new(25 + (25 + 150) * x_, 25 + (25 + 150) * y_, 150, 150);
}

pub fn main() {
    let mut game = game::Game::new();

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
                    } else {
                        game.place_card_at(0, 0).unwrap();
                    }
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0x60, 0x60, 0x60));
        canvas.clear();

        // Render current (drawn) card

        draw_card(&mut canvas, &texture_creator, &font, game.drawn(), (3, -2));

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if !((x == -2 || x == 2) && (y == -2 || y == 2)) {
                    draw_card(&mut canvas, &texture_creator, &font, game.get_card_at(x, y).unwrap(), (x, y));
                }
            }
        }

        canvas.present();
    }
}
