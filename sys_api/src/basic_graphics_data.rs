use glium::implement_vertex;

#[derive(Clone, Copy, Debug)]
pub struct GlVertex {
    pub pos : [f32; 2],
    pub tex_coord : [f32; 2],
}

implement_vertex!(GlVertex, pos, tex_coord);

/// A quad which is placed in the middle of the screen
pub const QUAD_VERTEX_DATA : [GlVertex; 6] = 
[
    GlVertex { pos : [-1.0f32, -1.0f32], tex_coord : [0.0f32, 0.0f32] },
    GlVertex { pos : [1.0f32, -1.0f32], tex_coord : [1.0f32, 0.0f32] },
    GlVertex { pos : [-1.0f32, 1.0f32], tex_coord : [0.0f32, 1.0f32] },
    GlVertex { pos : [1.0f32, -1.0f32], tex_coord : [1.0f32, 0.0f32] },
    GlVertex { pos : [-1.0f32, 1.0f32], tex_coord : [0.0f32, 1.0f32] },
    GlVertex { pos : [1.0f32, 1.0f32], tex_coord : [1.0f32, 1.0f32] }
];

pub static SPRITE_VERTEX_SHADER : &'static str = include_str!("shaders/basic_vertex_shader.glsl");

pub static SPRITE_FRAGMENT_SHADER : &'static str = include_str!("shaders/basic_fragment_shader.glsl");

pub static BLUR_FRAGMENT_SHADER : &'static str = include_str!("shaders/blur_shader.glsl");

pub static INSTANCED_SPRITE_VERTEX_SHADER : &'static str = include_str!("shaders/instanced_sprite_vertex_shader.glsl");

include!(concat!(env!("OUT_DIR"), "/missing_tex.rs"));

#[derive(Clone, Copy, Debug)]
pub struct SpriteData {
    pub mat_col1 : [f32; 4],
    pub mat_col2 : [f32; 4],
    pub mat_col3 : [f32; 4],
    pub mat_col4 : [f32; 4],
    pub texture_bottom_left : [f32; 2],
    pub width_height : [f32; 2],
    pub color : [f32; 4],
}

pub const ZEROED_SPRITE_DATA : SpriteData =
    SpriteData {
        mat_col1 : [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        mat_col2 : [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        mat_col3 : [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        mat_col4 : [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        texture_bottom_left : [0.0f32, 0.0f32],
        width_height : [0.0f32, 0.0f32],
        color : [0.0f32, 0.0f32, 0.0f32, 0.0f32],
    }
;

implement_vertex!(SpriteData, mat_col1, mat_col2, mat_col3, mat_col4, texture_bottom_left, width_height, color);
