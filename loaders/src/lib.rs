use std::path::Path;
use std::error::Error as StdError;

use log::{ trace, error };
use glium::texture::texture2d::Texture2d;
use glium::texture::RawImage2d;
use glium::backend::Facade;

use pack::UvCoordTable;
use sys_api::basic_graphics_data::{ MISSING_TEXTURE_DATA, MISSING_TEXTURE_DIMENSIONS };

fn load_image_from_file<P : AsRef<Path>>(p : P) -> Result<image::RgbaImage, Box<dyn StdError>> {
    trace!("loading texture: \"{}\"", p.as_ref().to_str().unwrap_or("UNPRINTABLE_PATH"));

    let img = 
        match image::open(p) {
            Ok(x) => Some(x.into_rgba8()),
            Err(err) => { 
                error!("failed to load texture. Error:\n {}\nThe texture will be replaced with \"missing_tex\"", err); 
                None 
            },
        }
    ;
       
    Ok(
        match img {
            Some(x) => x,
            None =>
                image::DynamicImage::ImageRgb8(
                    image::RgbImage::from_raw(
                        MISSING_TEXTURE_DIMENSIONS.0, 
                        MISSING_TEXTURE_DIMENSIONS.1, 
                        MISSING_TEXTURE_DATA.to_vec()
                    )
                    .expect("FATAL ERROR: Missing texture construction failed")
                )
                .into_rgba8()
            ,
        }
    )
}

fn texture_from_image<F : Facade>(f : &F, img : &image::RgbaImage) -> Result<Texture2d, Box<dyn StdError>> {
    let dims = img.dimensions();
    let raw_img = RawImage2d::from_raw_rgba_reversed(&img, dims);

    trace!("forwarding to GL");
    Texture2d::new(f, raw_img).map_err(|e| e.into())
}

pub fn load_texture_from_file<F : Facade, P : AsRef<Path>>(f : &F, p : P) -> Result<Texture2d, Box<dyn StdError>> {
    load_image_from_file(p)
    .and_then(|x| texture_from_image(f, &x))
}

fn load_atlas_from_file<P : AsRef<Path>>(p : Vec<(String, P)>) -> Result<pack::Atlas<image::Rgba<u8>>, Box<dyn StdError>> {
    trace!("Loading the images");
    let images = 
        p.into_iter()
        .map(
            |(name, path)| -> Result<_, Box<dyn StdError>> { 
                Ok((name, load_image_from_file(path)?)) 
            }
        ).collect::<Result<Vec<_>, _>>()?
    ;

    let images = 
        images.into_iter()
        .map(
            |(name, img)| {
                (
                    name,
                    pack::Image {
                        width : img.width() as u64,
                        height : img.height() as u64,
                        data :
                            img.rows()
                            .map(|x| x.map(|y| *y).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                        ,
                    }
                )
            }
        )
        .collect()
    ;

    trace!("Packing");
    Ok(pack::pack(images, image::Rgba([0, 0, 0, 0])))
}

pub fn load_atlas_uv_from_file<F : Facade, P : AsRef<Path>>(f : &F, p : Vec<(String, P)>) -> Result<(Texture2d, UvCoordTable), Box<dyn StdError>> {
    let atlas = load_atlas_from_file(p)?;
    let img = 
        image::ImageBuffer::<image::Rgba<u8>, _>::from_vec(
            atlas.width as u32,
            atlas.height as u32,
            atlas.pixels.iter().flat_map(|x| x.0.iter().map(|x| *x)).collect(),
        ).expect("Bad atlas image")
    ;
    let texture = texture_from_image(f, &img)?;
    let uv_table = UvCoordTable::create(atlas);
    Ok((texture, uv_table))
}
