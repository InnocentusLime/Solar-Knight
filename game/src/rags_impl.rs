use rags::{ GraphicsBackend };
use sys_api::{ graphics_init::GraphicsContext };

use cgmath::{ Matrix4, EuclideanSpace, vec2, conv::array4x4 };
use glium::{ Frame, DrawParameters, Rect, Blend, Surface, index, uniform };

pub struct RagsBackend<'a, 'b> {
    width : u32,
    height : u32,
    frame : &'a mut Frame,
    ctx : &'a mut GraphicsContext,
    params : DrawParameters<'b>,
}

impl<'a, 'b> RagsBackend<'a, 'b> {
    pub fn new(frame : &'a mut Frame, ctx : &'a mut GraphicsContext) -> Self {
        let mut params = DrawParameters::default();
        params.blend = Blend::alpha_blending();
        let (width, height) = frame.get_dimensions();

        RagsBackend {
            width,
            height,
            frame,
            ctx,
            params,
        }
    }
}

impl<'a, 'b> GraphicsBackend for RagsBackend<'a, 'b> {
    fn scissor(&mut self, (left, top) : (u32, u32), (width, height) : (u32, u32)) {
        self.params.scissor = Some (Rect { left, bottom : top + height, width, height });
    }

    fn draw_box(&mut self, (left, top) : (i32, i32), (width, height) : (u32, u32), (r, g, b, a) : (u8, u8, u8, u8)) {
        use cgmath::ortho;
        use sys_api::graphics_init::SCREEN_WIDTH;

        let (left, top) = (left as f32 / self.width as f32, top as f32 / self.height as f32);
        let (width, height) = (width as f32 / self.width as f32, height as f32 / self.height as f32);
        let (r, g, b, a) = (r as f32 / 255.0f32, g as f32 / 255.0f32, b as f32 / 255.0f32, a as f32 / 255.0f32);

        let mvp =
            ortho(0.0f32, SCREEN_WIDTH, 1.0f32, 0.0f32, -1.0f32, 1.0f32) *
            Matrix4::from_translation(vec2(left, top).extend(0.0f32)) * 
            Matrix4::from_nonuniform_scale(width, height, 1.0f32)
        ;
    
        let uniforms =
            uniform!(
                mvp : array4x4(mvp),
                col : [r, g, b, a],
            )
        ;

        self.frame.draw(
            &self.ctx.quad_vertex_buffer,
            &index::NoIndices(index::PrimitiveType::TrianglesList),
            &self.ctx.solid_shader,
            &uniforms,
            &self.params,
        ).unwrap();
    }
}
