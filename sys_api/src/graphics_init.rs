use std::error::Error as StdError;

use glium::glutin;
use glium::texture::texture2d::Texture2d;
use glium::{ Display, VertexBuffer, Program, Rect };
use glutin::dpi::{ Size, LogicalSize, PhysicalSize };
use glutin::{ ContextBuilder, event_loop::EventLoop };
use glium::glutin::window::WindowBuilder;
use glium::debug::DebugCallbackBehavior;
use glium::buffer::WriteMapping;
use nalgebra::{ Matrix4, Isometry3 };

use crate::verbose_try;
use crate::basic_graphics_data::*;

const WINDOW_TITLE : &'static str = "solar knight";
const WINDOW_DEFAULT_WIDTH : f64 = 1280.0f64;
const WINDOW_DEFAULT_HEIGHT : f64 = 720.0f64;
const WINDOW_SIZABLE : bool = false;

pub const ASPECT_RATIO : f32 = 16.0f32 / 9.0f32;
pub const SCALING : f32 = 1.0f32;
pub const SCREEN_WIDTH : f32 = ASPECT_RATIO * SCALING;
pub const SCREEN_RIGHT : f32 = SCREEN_WIDTH;
pub const SCREEN_LEFT : f32 = -SCREEN_RIGHT;

pub const SPRITE_DEBUG_LIMIT : usize = 10;
pub const ENEMY_BULLET_LIMIT : usize = 10_000;
pub const ENEMY_LIMIT : usize = 1000;
pub const PLAYER_BULLET_LIMIT : usize = ENEMY_BULLET_LIMIT;

pub struct SpriteDataWriter<'a> {
    id : usize,
    map : WriteMapping<'a, [SpriteData]>,
}

impl<'a> SpriteDataWriter<'a> {
    #[inline]
    pub fn new(map : WriteMapping<'a, [SpriteData]>) -> Self {
        SpriteDataWriter {
            id : 0,
            map,
        }
    }

    pub fn put(&mut self, x : SpriteData) {
        if self.id < self.map.len() {
            self.map.set(self.id, x);
            self.id += 1;
        } else {
            #[cfg(debug_assertions)]
            panic!("Vertex buffer overflow.");
        }
    }
}

pub struct RenderTargets {
    pub framebuffer1 : Texture2d,
}

/// This is the structure which will hold vital graphics resources required for
/// the startup.
pub struct GraphicsContext {
    pub display : Display,
    pub camera : Isometry3<f32>,
    pub proj_mat : Matrix4<f32>,
    pub quad_vertex_buffer : VertexBuffer<GlVertex>,
    pub sprite_shader : Program,
    pub blur_shader : Program,
    pub solid_shader : Program,
    pub instanced_sprite_shader : Program,
    pub bullet_buffer : VertexBuffer<SpriteData>, 
    pub enemy_buffer : VertexBuffer<SpriteData>,
    pub sprite_debug_buffer : VertexBuffer<SpriteData>,
}

impl GraphicsContext {
    #[inline]
    pub fn physical_size_defualt(&self) -> PhysicalSize<u32> {
        let dpi = self.display.gl_window().window().scale_factor();
        LogicalSize::new(
            WINDOW_DEFAULT_WIDTH,
            WINDOW_DEFAULT_HEIGHT
        ).to_physical(dpi)
    }

    #[inline]
    pub fn viewport(&self) -> Rect {
        let window_size = self.display.gl_window().window().inner_size();
        let view_width = (window_size.height as f32 * ASPECT_RATIO) as u32;
        let left = (window_size.width - view_width) / 2;
        Rect {
            left : left,
            bottom : 0,
            width : view_width,
            height : window_size.height,
        }
    }

    pub fn set_window_size<S : Into<Size>>(&mut self, new_size : S) {
        let gl_window = self.display.gl_window();
        let window = gl_window.window();
        window.set_inner_size(new_size);
    }

    #[inline]
    pub fn build_projection_view_matrix(&self) -> Matrix4<f32> {
        self.proj_mat * self.camera.to_homogeneous()
    }

    pub fn new() -> Result<(GraphicsContext, EventLoop<()>, RenderTargets), Box<dyn StdError>> {
        let event_loop = glutin::event_loop::EventLoop::new();
        
        let wb = 
            WindowBuilder::new()
            .with_resizable(WINDOW_SIZABLE)
            .with_inner_size(
                Size::Logical(
                    LogicalSize::new(
                        WINDOW_DEFAULT_WIDTH, 
                        WINDOW_DEFAULT_HEIGHT
                    )
                )
            )
            .with_title(WINDOW_TITLE)
        ;

        let cb = 
            ContextBuilder::new()
            .with_vsync(true)
        ;

        let windowed_ctx = cb.build_windowed(wb, &event_loop)?;
        
        let display = verbose_try!(Display::with_debug(windowed_ctx, DebugCallbackBehavior::PrintAll), display_creation);

        let phys_size = {
            let dpi = display.gl_window().window().scale_factor();
            LogicalSize::new(
                WINDOW_DEFAULT_WIDTH,
                WINDOW_DEFAULT_HEIGHT
            ).to_physical(dpi)
        };

        // I like when when a quad with vertices (0,0), (1,0), (1,1) and (0,1) forms a square.
        // So I create a projection matrix for that.
        // This matrix is purely for projecting things that exists on the level. Hence the origin
        // (the (0, 0) point) is in the center of the screen. It will be easier for cameras and
        // stuff. 
        // IMHO, an orign which is in top left corner is more sensible for an UI rather than for
        // level geometry
        let proj_mat = Matrix4::new_orthographic(SCREEN_LEFT, SCREEN_RIGHT, -1.0f32, 1.0f32, -1.0f32, 1.0f32);
        //let proj_mat = ortho(2.0f32*SCREEN_LEFT, 2.0f32*SCREEN_RIGHT, -2.0f32, 2.0f32, -1.0f32, 1.0f32);

        let framebuffer1 = verbose_try!(Texture2d::empty(&display, phys_size.width, phys_size.height), framebuffer1_creation);

        let quad_vertex_buffer = verbose_try!(VertexBuffer::immutable(&display, &QUAD_VERTEX_DATA), quad_buffer_creation);

        let sprite_shader = 
            verbose_try!(
                Program::from_source(
                    &display, 
                    SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER, 
                    None
                ), 
                basic_shader_creation
            )
        ;

        let blur_shader =
            verbose_try!(
                Program::from_source(
                    &display,
                    SPRITE_VERTEX_SHADER, BLUR_FRAGMENT_SHADER,
                    None
                ),
                blur_shader_creation
            )
        ;

        let instanced_sprite_shader = 
            verbose_try!(
                Program::from_source(
                    &display, 
                    INSTANCED_SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER, 
                    None
                ), 
                instanced_shader_creation
            )
        ;
        
        let solid_shader = 
            verbose_try!(
                Program::from_source(
                    &display, 
                    SOLID_VERTEX_SHADER, SOLID_FRAGMENT_SHADER, 
                    None
                ), 
                solid_shader_creation
            )
        ;

        let bullet_buffer =
            verbose_try!(
                VertexBuffer::dynamic(&display, &[ZEROED_SPRITE_DATA; PLAYER_BULLET_LIMIT]),
                player_bullet_buffer
            )
        ;
        
        let enemy_buffer =
            verbose_try!(
                VertexBuffer::dynamic(&display, &[ZEROED_SPRITE_DATA; ENEMY_LIMIT]),
                enemy_buffer
            )
        ;
        
        let sprite_debug_buffer =
            verbose_try!(
                VertexBuffer::dynamic(&display, &[ZEROED_SPRITE_DATA; SPRITE_DEBUG_LIMIT]),
                sprite_debug_buffer
            )
        ;
        
        Ok(
            (
                GraphicsContext {
                    display,
                    camera : Isometry3::identity(),
                    proj_mat,
                    quad_vertex_buffer,
                    sprite_shader,
                    blur_shader,
                    instanced_sprite_shader,
                    solid_shader,
                    bullet_buffer,
                    enemy_buffer,
                    sprite_debug_buffer,
                },
                event_loop,
                RenderTargets {
                    framebuffer1,
                }
            )
        )
    }
}
