use glium::VertexBuffer;
use glium::backend::Facade;
use glium::texture::{ Texture2d, RawImage2d };
use std::collections::HashMap;

use crate::pack::{ Atlas, GlyphData };

use sys_api::basic_graphics_data::SpriteData;

struct TextPositioningContext {
    start_x : u32,
    start_y : u32,
    curr_x : u32,
    curr_y : u32,
}

pub struct Font {
    glyph_data : HashMap<char, GlyphData>,
    texture : Texture2d,
}

impl Font {
    // 1. Take the set of glyphs
    // 2. pack it, create the texture and init
    pub fn new<F : Facade>(atlas : Atlas, f : &F) -> Self {
        // FIXME this is porbably bad. I am doing this purely
        // because glium is having problems with getting
        // different formats. I should either fix it or
        // find a better "backend"
        
        // Waiting for 1.53
        use std::array::IntoIter;
        let img_as_lumalpha : Vec<_> =
            atlas.luma
            .into_iter()
            .flat_map(|x| IntoIter::new([x, x, x, x]))
            .collect()
        ;

        let raw_img = RawImage2d::from_raw_rgba_reversed(img_as_lumalpha.as_slice(), (atlas.width as u32, atlas.height as u32));
    
        let texture = Texture2d::new(f, raw_img).expect("OpenGL failed to allocate texture");

        Font {
            glyph_data : atlas.glyph_data,
            texture,
        }
    }

    fn put_char(&self, ch : char, x : &mut u32, y : &mut u32) {
        match (ch, self.glyph_data.get(&ch)) {
            ('\n', _) => todo!("TODO: carriage return"),
            (_, Some(ch)) => {
                unimplemented!()
            },
            (_, None) => panic!("Character '{}' isn't supported", ch),
        }
    }
    
    pub fn fill_buffer(&self, buff : &mut VertexBuffer<SpriteData>, text : &str) {
        // Normalize all the shit and render it
        // Size stuff: 'A' is treated as a character with height equal to `1`
        // All stuff can be scaled into what we need with matrices, so yeah
        let mut ptr = buff.map_write();

        for i in 0..ptr.len() { 
            use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        /*
        text.chars()
        .for_each(|x| )
        */
    }
}
