mod impl_prelude {
    pub use bevy_inspector_egui::options::*;
    pub use bevy_inspector_egui::Inspectable;
    pub use bevy_inspector_egui::egui::Ui;
    pub use bevy_inspector_egui::Context;
}

pub mod nalgebra {
    use super::impl_prelude::*;
    use bevy_rapier2d::prelude::nalgebra;

    pub fn inspect_vec2(
        v : &mut nalgebra::Vector2<f32>,
        ui : &mut Ui,
        attributes : Vec2dAttributes,
        ctx : &mut Context,
    ) -> bool {
        use bevy::prelude::Vec2;

        let mut changed = false;

        let v = &mut (v.data.0)[0]; 
        let mut buff = Vec2::new(v[0], v[1]);
        changed |= buff.ui(ui, attributes, ctx);
        *v = buff.to_array();

        changed
    }
}
