use glium::texture::Texture2d;
use std::collections::HashMap;

use crate::pack::{ Atlas, GlyphData };

pub struct Font {
    width : usize,
    height : usize,
    glyph_data : HashMap<char, GlyphData>,
    texture : Texture2d,
}

impl Font {
    // 1. Take the set of glyphs
    // 2. pack it, create the texture and init
    pub fn new() -> Self { todo!() }
    
    pub fn fill_buffer() {
        // Normalize all the shit and render it
        // Size stuff: 'A' is treated as a character with height equal to `1`
        // All stuff can be scaled into what we need with matrices, so yeah
    }
}
