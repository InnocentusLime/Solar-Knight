use std::fmt;
use std::collections::HashMap;

use crate::freetype_wrap::GlyphImage;

// TODO polish

pub struct GlyphData {
    pub width : usize,
    pub height : usize,
    pub bearing : (i32, i32),
    pub advance : u64,
    pub off_x : usize,
    pub off_y : usize,
}

pub struct Atlas {
    pub glyph_data : HashMap<char, GlyphData>,
    pub luma : Vec<u8>,
    pub width : usize,
    pub height : usize,
}

struct MultiDim<T> {
    v : Vec<Vec<T>>,
}

impl<T : Copy + fmt::Display> MultiDim<T> {
    fn new() -> MultiDim<T> {
        MultiDim {
            v : vec![],
        } 
    }

    fn width(&self) -> usize { self.v[0].len() }

    fn height(&self) -> usize { self.v.len() }

    fn resize(&mut self, width : usize, height : usize, val : T) {
        self.v.resize(height, vec![val; width]);
        self.v.iter_mut().for_each(|v| v.resize(width, val));
    }

    fn copy(&mut self, data : &Vec<Vec<T>>, off_x : usize, off_y : usize) {
        data.iter()
        .enumerate()
        .for_each(
            |(y, row)| {
                row.iter()
                .enumerate()
                .for_each(
                    |(x, pix)| {
                        self.v[y + off_y][x + off_x] = *pix
                    }
                )
            }
        )
    }

    fn set(&mut self, val : T, off_x : usize, off_y : usize, width : usize, height : usize) {
        (off_y..off_y+height)
        .for_each(
            |y| {
                (off_x..off_x+width)
                .for_each(
                    |x| 
                    self.v[y][x] = val
                )
            }
        )
    }
}

// A port of
// https://github.com/zorbathut/glorp/blob/4307a13af75ca1c5386988b1b693c5d97a4c3a94/fontbaker/main.cpp
// Supposed to be a relatively good text atlas creator
// TODO try to clean it
// TODO make `WIDTH_CAP` an argument
// TODO make the impl look cleaner
// TODO replace some panics with error codes
pub fn pack(mut glyphs : Vec<(char, GlyphImage)>) -> Atlas {
    // TODO can be changed to max width of the glyph, I guess...
    const WIDTH_CAP : usize = 512;

    glyphs.sort_by_key(|x| x.1.width * x.1.height);
    glyphs.reverse();

    let mut dest : MultiDim<u8> = MultiDim::new();
    let mut mask : MultiDim<bool> = MultiDim::new();
    let mut glyph_data = HashMap::new();
    for glyph in glyphs {
        let (glyph_width, glyph_height) = (glyph.1.width as usize, glyph.1.height as usize);
        
        assert!(glyph_width <= WIDTH_CAP, "The width of glyph `{}` is too big", glyph.0);
        let success =
            (0..2048)
            .any(
                |ty : usize| {
                    if ty + glyph_height > dest.height() {
                        dest.resize(WIDTH_CAP, ty + glyph_height, 0);
                        mask.resize(WIDTH_CAP, ty + glyph_height, false);
                    }

                    (0..=(WIDTH_CAP - glyph_width))
                    .any(
                        |tx : usize| {
                            let mut valid = 
                                !(
                                    mask.v[ty][tx] || 
                                    mask.v[ty + glyph_height - 1][tx] ||
                                    mask.v[ty][tx + glyph_width - 1] ||
                                    mask.v[ty + glyph_height - 1][tx + glyph_width - 1]
                                )
                            ;
    
                            if !valid { return false; }

                            if 
                                (0..glyph_height)
                                .any(
                                    |ity : usize| 
                                    (0..glyph_width)
                                    .any(|itx : usize| mask.v[ity + ty][itx + tx])
                                )
                            { return false; }

                            dest.copy(&glyph.1.data, tx, ty);
                            mask.set(true, tx, ty, glyph_width, glyph_height);

                            glyph_data.insert(
                                glyph.0, 
                                GlyphData {
                                    width : glyph_width,
                                    height : glyph_height,
                                    bearing : glyph.1.bearing,
                                    advance : glyph.1.advance,
                                    off_x : tx,
                                    off_y : ty,
                                }
                            );

                            //println!("Put `{}` at ({}, {})-({}, {})", glyph.0, tx, ty, tx+glyph_width, ty+glyph_height);
    
                            true
                        }
                    )
                }
            )
        ;

        assert!(success, "Failed to lay in glyph `{}`", glyph.0);
    }

    Atlas {
        width : dest.width(),
        height : dest.height(),
        luma : dest.v.into_iter().flatten().collect(),
        glyph_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freetype_wrap::*;

    #[test]
    fn test_bake_charset() {
        use core::num::NonZeroU32;

        let lib = Lib::new().unwrap();
        let face = 
            Face::new(
                &lib, 
                NonZeroU32::new(0), NonZeroU32::new(48),
                "fonts/OpenSans-Regular.ttf"
            ).unwrap()
        ;

        let arr = 
        [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
            'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
            'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '+', '=',
            '[', ']', '{', '}', ':', ';', '\'', '"', ',', '.', '/', '<', '>', '?',
            '\\', '|', '`', '~'
        ];

        let input = 
            arr.iter()
            .map(
                |ch| {
                    let glyph = 
                        face.load_char(*ch)
                        .expect(&format!("Failed to load char `{}`", ch))
                    ; 
                    (*ch, glyph)
                }
            )
            .collect()
        ;

        pack(input);
    }
}
