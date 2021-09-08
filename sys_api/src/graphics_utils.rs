use glium::{ draw_parameters, index, VertexBuffer, Surface, Blend, Rect, uniform };
use glium::texture::texture2d::Texture2d;
use glium::uniforms::Sampler;
use nalgebra::Matrix4;

use crate::basic_graphics_data::SpriteData;
use crate::graphics_init::GraphicsContext;

pub fn draw_sprite<S : Surface>(
    ctx : &GraphicsContext, 
    target : &mut S, 
    mvp : Matrix4<f32>, 
    tex_view : (f32, f32, f32, f32),
    texture : Sampler<Texture2d>, 
    viewport : Option<Rect>
) {
    use glium::uniforms::{ MinifySamplerFilter, MagnifySamplerFilter };

    let mut draw_params = draw_parameters::DrawParameters::default();
    draw_params.blend = Blend::alpha_blending();
    draw_params.viewport = viewport;

    let uniforms =
        uniform!(
            tex : texture.minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
            mvp : mvp.data.0,
            texture_bottom_left : [tex_view.0, tex_view.1],
            width_height : [tex_view.2, tex_view.3],
        )
    ;

    target.draw(
        &ctx.quad_vertex_buffer,
        &index::NoIndices(index::PrimitiveType::TrianglesList),
        &ctx.sprite_shader,
        &uniforms,
        &draw_params,
    ).unwrap();
}

pub fn draw_instanced_sprite<S : Surface>(ctx : &GraphicsContext, target : &mut S, instance_data : &VertexBuffer<SpriteData>, vp : Matrix4<f32>, texture : Sampler<Texture2d>, viewport : Option<Rect>) {
    use glium::uniforms::{ MinifySamplerFilter, MagnifySamplerFilter };

    let mut draw_params = draw_parameters::DrawParameters::default();
    draw_params.blend = Blend::alpha_blending();
    draw_params.viewport = viewport;

    let uniforms =
        uniform!(
            tex : texture.minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
            vp : vp.data.0,
        )
    ;

    target.draw(
        (&ctx.quad_vertex_buffer, instance_data.per_instance().unwrap()),
        &index::NoIndices(index::PrimitiveType::TrianglesList),
        &ctx.instanced_sprite_shader,
        &uniforms,
        &draw_params,
    ).unwrap();
}
