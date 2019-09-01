use sdl2::rect::Rect;
use sdl2::ttf::Font;

pub enum HorizontalAlignment {
    Left, Center, Right,
}

pub enum VerticalAlignment {
    Top, Middle, Bottom,
}

pub fn align_text<'a>(font: &Font<'a, 'static>, text: &'a str, parent: Rect, horizontal_alignment: HorizontalAlignment, vertical_alignment: VerticalAlignment, horizontal_margin: i32, vertical_margin: i32) -> (i32, i32) {
    use HorizontalAlignment::*;
    use VerticalAlignment::*;

    let (tw_, th_) = font.size_of(text).unwrap_or((0, 0));
    let (tw, th) = (tw_ as i32, th_ as i32);
    let (px, py, pw, ph) = (parent.x(), parent.y(), parent.width() as i32, parent.height() as i32);

    let x = match horizontal_alignment {
        Left => px + horizontal_margin,
        Center => px + (pw - tw) / 2 - 1,  // Margin ignored
        Right => px + pw - tw - horizontal_margin,
    };

    let y = match vertical_alignment {
        Top => py + vertical_margin,
        Middle => py + (ph - th) / 2,
        Bottom => py + ph - th - vertical_margin,  // Margin ignored
    };

    (x, y)
}

