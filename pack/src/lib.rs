use std::collections::HashMap;

pub struct ImageData {
    pub width : usize,
    pub height : usize,
    pub off_x : usize,
    pub off_y : usize,
}

pub struct Image<T> {
    pub width : u64,
    pub height : u64,
    pub data : Vec<Vec<T>>,
}

pub struct Atlas<T> {
    pub img_data : HashMap<String, ImageData>,
    pub pixels : Vec<T>,
    pub width : usize,
    pub height : usize,
}

struct MultiDim<T : Copy> {
    v : Vec<Vec<T>>,
}

impl<T : Copy> MultiDim<T> {
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
pub fn pack<T : Copy + Default>(mut glyphs : Vec<(String, Image<T>)>) -> Atlas<T> {
    // TODO can be changed to max width of the glyph, I guess...
    const WIDTH_CAP : usize = 512;

    use log::trace;

    glyphs.sort_by_key(|x| x.1.width * x.1.height);
    glyphs.reverse();

    let mut dest : MultiDim<T> = MultiDim::new();
    let mut mask : MultiDim<bool> = MultiDim::new();
    let mut img_data = HashMap::new();
    for glyph in glyphs {
        let (img_width, img_height) = (glyph.1.width as usize, glyph.1.height as usize);
        
        assert!(img_width <= WIDTH_CAP, "The width of image `{}` is too big", glyph.0);
        let success =
            (0..2048)
            .any(
                |ty : usize| {
                    if ty + img_height > dest.height() {
                        dest.resize(WIDTH_CAP, ty + img_height, T::default());
                        mask.resize(WIDTH_CAP, ty + img_height, false);
                    }

                    (0..=(WIDTH_CAP - img_width))
                    .any(
                        |tx : usize| {
                            let mut valid = 
                                !(
                                    mask.v[ty][tx] || 
                                    mask.v[ty + img_height - 1][tx] ||
                                    mask.v[ty][tx + img_width - 1] ||
                                    mask.v[ty + img_height - 1][tx + img_width - 1]
                                )
                            ;
    
                            if !valid { return false; }

                            if 
                                (0..img_height)
                                .any(
                                    |ity : usize| 
                                    (0..img_width)
                                    .any(|itx : usize| mask.v[ity + ty][itx + tx])
                                )
                            { return false; }

                            dest.copy(&glyph.1.data, tx, ty);
                            mask.set(true, tx, ty, img_width, img_height);

                            img_data.insert(
                                glyph.0.clone(), 
                                ImageData {
                                    width : img_width,
                                    height : img_height,
                                    off_x : tx,
                                    off_y : ty,
                                }
                            );

                            trace!("Put `{}` at ({}, {})-({}, {})", glyph.0, tx, ty, tx + img_width, ty + img_height);
    
                            true
                        }
                    )
                }
            )
        ;

        assert!(success, "Failed to lay in image `{}`", glyph.0);
    }

    Atlas {
        width : dest.width(),
        height : dest.height(),
        pixels : dest.v.into_iter().flatten().collect(),
        img_data,
    }
}

// Uv coordinates of an image.
// The coordinate system encoded by this
// struct is OpenGL-like
#[derive(Clone, Copy)]
pub struct UvCoordTableEntry {
    pub left : f32,
    pub bottom : f32,
    pub right : f32,
    pub top : f32,
}

pub struct UvCoordTable {
    pub entries : HashMap<String, UvCoordTableEntry>,
}

impl UvCoordTable {
    pub fn create<T>(atlas : Atlas<T>) -> Self {
        UvCoordTable {
            entries : 
            atlas.img_data.into_iter()
            .map(|(name, entry)| {
                (
                    name, 
                    UvCoordTableEntry {
                        left : (entry.off_x as f32) / ((entry.width - 1) as f32),
                        bottom : ((entry.height - entry.off_y - 1) as f32) / ((entry.height - 1) as f32),
                        right : ((entry.width - entry.off_x - 1) as f32) / ((entry.width - 1) as f32),
                        top : (entry.off_y as f32) / ((entry.height - 1) as f32),
                    }
                )
            }).collect()
        }
    }
}
