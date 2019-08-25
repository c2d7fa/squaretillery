extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use sdl2::ttf::{Font};

fn draw_text<'a, T: RenderTarget, U>(canvas: &mut Canvas<T>, texture_creator: &TextureCreator<U>, text: &'a str, font: &Font, coord: (i32, i32))  {
    let text_size = font.size_of(text).unwrap();
    let text_surface = font.render(text).blended(Color::RGB(0xFF, 0xFF, 0xFF)).unwrap();
    let text_texture = texture_creator.create_texture_from_surface(text_surface).unwrap();
    canvas.copy(&text_texture, None, Rect::new(coord.0, coord.1, text_size.0, text_size.1)).unwrap();
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let sdl_video = sdl_context.video().unwrap();
    let sdl_ttf = sdl2::ttf::init().unwrap();

    let mono = sdl_ttf.load_font("./mono.ttf", 22).unwrap();
    let sans = sdl_ttf.load_font("./sans.ttf", 22).unwrap();

    let window = sdl_video.window("Rust SDL2 demo", 1600, 800)
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

        draw_text(&mut canvas, &texture_creator, "Hello, world!", &mono, (50, 50));
        draw_text(&mut canvas, &texture_creator, "This is a second line of text.", &mono, (50, 50 + 24));
        draw_text(&mut canvas, &texture_creator, &format!("i = {}", i), &mono, (50, 50 + 24 * 2));
        if buffer.len() > 0 {
            draw_text(&mut canvas, &texture_creator, &buffer, &sans, (50, 50 + 24 * 4));
        }

        // The rest of the game loop goes here.

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
