use std::path::Path;
use std::error::Error as StdError;

use log::{ trace, error };
use glium::texture::texture2d::Texture2d;
use glium::texture::RawImage2d;
use glium::backend::Facade;

use crate::verbose_try;
use crate::basic_graphics_data::{ MISSING_TEXTURE_DATA, MISSING_TEXTURE_DIMENSIONS };

pub fn load_from_file<F : Facade, P : AsRef<Path>>(f : &F, p : P) -> Result<Texture2d, Box<dyn StdError>> {
    trace!(target: "load_texture", "loading texture: \"{}\"", p.as_ref().to_str().unwrap_or("UNPRINTABLE_PATH"));

    let img = 
        match image::open(p) {
            Ok(x) => Some(x.to_rgba()),
            Err(err) => { error!(target : "load_texture", "failed to load texture. Error:\n {}\nThe texture will be replaced with \"missing_tex\"", err); None },
        }
    ;
    let raw_img = 
        match img {
            Some(x) => RawImage2d::from_raw_rgba_reversed(&x, x.dimensions()),
            None => RawImage2d::from_raw_rgb_reversed(&MISSING_TEXTURE_DATA, MISSING_TEXTURE_DIMENSIONS),
        }
    ;
    trace!(target: "load_texture", "forwarding to GL");

    let tex = verbose_try!(Texture2d::new(f, raw_img), texture_creation);
    trace!(target: "load_texture", "Success");

    Ok(tex)
}

