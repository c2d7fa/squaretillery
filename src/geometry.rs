use sdl2::rect::Rect;
use sdl2::ttf::Font;

#[allow(dead_code)]
pub enum HorizontalAlignment {
    Left, Center, Right,
}

#[allow(dead_code)]
pub enum VerticalAlignment {
    Top, Middle, Bottom,
}

pub fn align_text<'a>(font: &Font<'a, 'static>, text: &'a str, parent: Rect, horizontal_alignment: HorizontalAlignment, vertical_alignment: VerticalAlignment, horizontal_margin: i32, vertical_margin: i32) -> (i32, i32) {
    use HorizontalAlignment::*;
    use VerticalAlignment::*;

    let (tw, th) = font.size_of(text).unwrap_or((0, 0));
    let (px, py, pw, ph) = (parent.x(), parent.y(), parent.width(), parent.height());

    let x = match horizontal_alignment {
        Left => px + horizontal_margin,
        Center => px + (pw - tw) as i32 / 2,  // Margin ignored
        Right => px + (pw - tw) as i32 - horizontal_margin,
    };

    let y = match vertical_alignment {
        Top => py + vertical_margin,
        Middle => py + (ph - th) as i32 / 2 + vertical_margin,
        Bottom => py + (ph - th) as i32 - vertical_margin,
    };

    (x, y)
}

