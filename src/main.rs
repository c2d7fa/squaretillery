use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use sdl2::ttf::{Font};

mod game;

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

fn draw_card(context: &mut DrawContext, card: Option<game::Card>, (x, y): (i8, i8)) {
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
                    } else {
                        game.place_card_at(0, 0).unwrap();
                    }
                },
                _ => {}
            }
        }

        context.canvas.set_draw_color(Color::RGB(0x60, 0x60, 0x60));
        context.canvas.clear();

        // Render current (drawn) card

        draw_card(&mut context, game.drawn(), (3, -2));

        // Render board

        for x in -2..(2 + 1) {
            for y in -2..(2 + 1) {
                if !((x == -2 || x == 2) && (y == -2 || y == 2)) {
                    draw_card(&mut context, game.get_card_at(x, y).unwrap(), (x, y));
                }
            }
        }

        context.canvas.present();
    }
}
