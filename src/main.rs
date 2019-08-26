extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use sdl2::ttf::{Font};

mod game;

fn draw_text<'a, T: RenderTarget, U>(canvas: &mut Canvas<T>, texture_creator: &TextureCreator<U>, text: &'a str, font: &Font, coord: (i32, i32))  {
    let text_size = font.size_of(text).unwrap();
    let text_surface = font.render(text).blended(Color::RGB(0x00, 0x00, 0x00)).unwrap();
    let text_texture = texture_creator.create_texture_from_surface(text_surface).unwrap();
    canvas.copy(&text_texture, None, Rect::new(coord.0, coord.1, text_size.0, text_size.1)).unwrap();
}

fn draw_card<'a, T: RenderTarget, U>(canvas: &mut Canvas<T>, texture_creator: &TextureCreator<U>, font: &Font, game: &game::Game, (x, y): (i8, i8)) -> Result<(), String> {
    return game.get_card_at(x, y).map(|it| { match it {
        Some(card) => {
            let rect = rect_for_card((x, y));
            canvas.fill_rect(rect).unwrap();
            draw_text(canvas, texture_creator, &format!("{}", card.format_text_short()), font, (rect.x() + 5, rect.y() + 5));
        },
        None => {
            let rect = rect_for_card((x, y));
            draw_text(canvas, texture_creator, "No card", font, (rect.x(), rect.y()));
        },
    }});
}

fn rect_for_card((x, y): (i8, i8)) -> Rect {
    let (x_, y_) = (x as i32 + 2, y as i32 + 2);
    return Rect::new(25 + (25 + 150) * x_, 25 + (25 + 150) * y_, 150, 150);
}

pub fn main() {
    let mut game = game::Game::new();

    // Graphics

    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();
    let sdl_ttf = sdl2::ttf::init().unwrap();

    let mono = sdl_ttf.load_font("./mono.ttf", 22).unwrap();
    //let sans = sdl_ttf.load_font("./sans.ttf", 22).unwrap();

    let window = sdl_video.window("Rust SDL2 demo", 925, 925)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut buffer = String::new();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 0xFF;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    buffer.pop();
                },
                Event::TextInput { text, .. } => {
                    buffer.push_str(&text);
                },
                _ => {}
            }
        }


        canvas.set_draw_color(Color::RGB(i, 0x80, 0xFF - i));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if !((x == -2 || x == 2) && (y == -2 || y == 2)) {
                    draw_card(&mut canvas, &texture_creator, &mono, &game, (x, y)).unwrap();
                }
            }
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
