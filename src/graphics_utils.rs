use glium::{ draw_parameters, index, VertexBuffer, Surface, Blend, Rect, uniform };
use glium::texture::texture2d::Texture2d;
use cgmath::{ Matrix4, Vector2 };

use crate::basic_graphics_data::SpriteData;
use crate::graphics_init::{ GraphicsContext, ASPECT_RATIO };

pub fn draw_sprite<S : Surface>(ctx : &GraphicsContext, target : &mut S, mvp : Matrix4<f32>, texture : &Texture2d, size : (f32, f32), viewport : Option<Rect>) {
    use glium::uniforms::{ MinifySamplerFilter, MagnifySamplerFilter };
    use cgmath::conv::{ array2, array4x4 };

    let mut draw_params = draw_parameters::DrawParameters::default();
    draw_params.blend = Blend::alpha_blending();
    draw_params.viewport = viewport;

    let uniforms =
        uniform!(
            tex : texture.sampled().minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
            mvp : array4x4(mvp),
            scale : [size.0, size.1],
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

pub fn draw_instanced_sprite<S : Surface>(ctx : &GraphicsContext, target : &mut S, instance_data : &VertexBuffer<SpriteData>, vp : Matrix4<f32>, texture : &Texture2d, size : (f32, f32), viewport : Option<Rect>) {
    use glium::uniforms::{ MinifySamplerFilter, MagnifySamplerFilter };
    use cgmath::conv::{ array2, array4x4 };

    let mut draw_params = draw_parameters::DrawParameters::default();
    draw_params.blend = Blend::alpha_blending();
    draw_params.viewport = viewport;

    let uniforms =
        uniform!(
            tex : texture.sampled().minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
            vp : array4x4(vp),
            scale : [size.0, size.1],
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
