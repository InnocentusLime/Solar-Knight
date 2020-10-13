use std::env;
use std::fs;
use std::path::Path;

const POWER_OF_TWO : u32 = 5;

// Since `pow` is not `const fn` yet, I have to use
// some heap allocations. :(
fn compute_missing_tex_pixels() -> Vec<u8> {
    let side_len = 2_usize.pow(POWER_OF_TWO);
    let side_half = side_len / 2;

    // first the data is encoded as bool's
    let mut first_data = vec![vec![false; side_len]; side_len];

    for y in 0..side_len {
        for x in 0..side_len {
            if y < side_half && side_half <= x { 
                first_data[y][x] = true 
            }

            if side_half <= y && x < side_half {
                first_data[y][x] = true
            }
        }
    }

    first_data
    .into_iter()
    .flatten()
    .flat_map(|x| if x { [255, 0, 255].iter() } else { [0, 0, 0].iter() })
    .copied()
    .collect::<Vec<_>>()
}

fn write_missing_tex_file() {
    let side_len = 2_usize.pow(POWER_OF_TWO);
    let data = compute_missing_tex_pixels();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("missing_tex.rs");
    let src = 
        format!(
            "pub static MISSING_TEXTURE_DATA : [u8; {}] = {:?};\n
             pub const MISSING_TEXTURE_DIMENSIONS : (u32, u32) = ({}, {});
            ",
            data.len(),
            data,
            side_len,
            side_len,
        )
    ;
    fs::write(
        &dest_path,
        &src,
    ).unwrap();
}

fn main() {
    write_missing_tex_file();
}
