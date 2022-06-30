//! Utility / conveniece functions for commonly use graphical shapes

use scale::Pt;
use Point;

// PDF doesn't understand what a "circle" is, so we have to
// approximate it.
const C: f64 = 0.551915024494;

/// Calculates and returns the points for an approximated circle, given a radius and an
/// offset into the page from the lower left corner.
#[inline]
pub fn calculate_points_for_circle<P: Into<Pt>>(
    radius: P,
    offset_x: P,
    offset_y: P,
) -> Vec<(Point, bool)> {
    let (radius, offset_x, offset_y) = (radius.into(), offset_x.into(), offset_y.into());
    let radius = radius.0;

    let p10 = Point { x: Pt(0.0 * radius), y: Pt(1.0 * radius) };
    let p11 = Point { x: Pt(C   * radius), y: Pt(1.0 * radius) };
    let p12 = Point { x: Pt(1.0 * radius), y: Pt(C   * radius) };
    let p13 = Point { x: Pt(1.0 * radius), y: Pt(0.0 * radius) };

    let p20 = Point { x: Pt(1.0 * radius), y: Pt(0.0 * radius) };
    let p21 = Point { x: Pt(1.0 * radius), y: Pt(-C  * radius) };
    let p22 = Point { x: Pt(C   * radius), y: Pt(-1.0 * radius) };
    let p23 = Point { x: Pt(0.0 * radius), y: Pt(-1.0 * radius) };

    let p30 = Point { x: Pt(0.0 * radius),  y: Pt(-1.0 * radius) };
    let p31 = Point { x: Pt(-C  * radius),  y: Pt(-1.0 * radius) };
    let p32 = Point { x: Pt(-1.0 * radius), y: Pt(-C   * radius) };
    let p33 = Point { x: Pt(-1.0 * radius), y: Pt(0.0 * radius) };

    let p40 = Point { x: Pt(-1.0 * radius), y: Pt(0.0 * radius) };
    let p41 = Point { x: Pt(-1.0 * radius), y: Pt(C * radius) };
    let p42 = Point { x: Pt(-C * radius),   y: Pt(1.0  * radius) };
    let p43 = Point { x: Pt(0.0 * radius),  y: Pt(1.0 * radius) };

    let mut pts  = vec![(p10, true), (p11, true), (p12, true), (p13, false),
                        (p20, true), (p21, true), (p22, true), (p23, false),
                        (p30, true), (p31, true), (p32, true), (p33, false),
                        (p40, true), (p41, true), (p42, true), (p43, false),
    ];

    for &mut (ref mut p, _) in pts.iter_mut() {
        p.x.0 += offset_x.0;
        p.y.0 += offset_y.0;
    }

    pts
}

/// Calculates and returns the points for a rectangle, given a horizontal and vertical scale,
/// and an offset into the page from the lower left corner.
#[inline]
pub fn calculate_points_for_rect<P: Into<Pt>>(
    scale_x: P,
    scale_y: P,
    offset_x: P,
    offset_y: P,
) -> Vec<(Point, bool)> {
    let (scale_x, scale_y, offset_x, offset_y) = (
        scale_x.into(),
        scale_y.into(),
        offset_x.into(),
        offset_y.into(),
    );
    let top = Pt(offset_y.0 + (scale_y.0 / 2.0));
    let bottom = Pt(offset_y.0 - (scale_y.0 / 2.0));
    let left = Pt(offset_x.0 - (scale_x.0 / 2.0));
    let right = Pt(offset_x.0 + (scale_x.0 / 2.0));

    let top_left_pt = Point { x: left, y: top };
    let top_right_pt = Point { x: right, y: top };
    let bottom_right_pt = Point { x: right, y: bottom };
    let bottom_left_pt = Point { x: left, y: bottom };

    vec![(top_left_pt, false), (top_right_pt, false), (bottom_right_pt, false), (bottom_left_pt, false)]
}

use std::sync::atomic::{AtomicUsize, Ordering};

/// Since the random number generator doesn't have to be cryptographically secure
/// it doesn't make sense to import the entire rand library, so this is just a
/// xorshift pseudo-random function
static RAND_SEED: AtomicUsize = AtomicUsize::new(2100);

/// Xorshift-based random number generator. Impure function
pub(crate) fn rand() -> usize {
    let mut x = RAND_SEED.fetch_add(21, Ordering::SeqCst);
    #[cfg(target_pointer_width = "64")] {
        x ^= x << 21;
        x ^= x >> 35;
        x ^= x << 4;
        x
    }

    #[cfg(target_pointer_width = "32")] {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }
}

/// Returns a string with 32 random characters
pub(crate) fn random_character_string_32() -> String {

    const MAX_CHARS: usize = 32;
    let mut final_string = String::with_capacity(MAX_CHARS);
    let mut char_pos = 0;

    'outer: while char_pos < MAX_CHARS {
        let rand = format!("{}", rand());
        for ch in rand.chars() {
            if char_pos < MAX_CHARS {
                final_string.push(u8_to_char(ch.to_digit(10).unwrap() as u8));
                char_pos += 1;
            } else {
                break 'outer;
            }
        }
    }

    final_string
}

#[inline(always)]
fn u8_to_char(input: u8) -> char {
    ('A' as u8 + input) as char
}

pub fn text_bytes_for_font(text: &str, font: &rusttype::Font) -> Vec<u8> {
    use rusttype::Codepoint as Cp;

    let char_iter = text.chars();
    let mut list_gid = Vec::<u16>::new();

    for ch in char_iter {
        // note: font.glyph will panic if the character is \0
        // since that can't happen in Rust, I think we're safe here
        let glyph = font.glyph(Cp(ch as u32));
        list_gid.push(glyph.id().0 as u16);

        // todo - kerning !!
        // font.pair_kerning(scale, id, base_glyph.id());
    }

    list_gid
        .iter()
        .flat_map(|x| vec![(x >> 8) as u8, (x & 255) as u8])
        .collect::<Vec<u8>>()
}
