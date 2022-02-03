use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::{ EnumIter, IntoStaticStr };
use bevy_inspector_egui::{ Inspectable, Context, egui::Ui };

#[derive(Clone, Copy, Debug, Inspectable, EnumIter, IntoStaticStr)]
pub enum Layer {
    BackgroundLayer,
    ShipLayer,
}

impl Layer {
    fn into_global_offset(self) -> f32 { (self as u32) as f32 }
}

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct LayerComponent {
    pub layer : Layer,
    #[inspectable(min = 0.0f32, max = 0.9f32)]
    pub internal_offset : f32,
}

impl LayerComponent {
    pub fn into_global_offset(self) -> f32 {
        debug_assert!(0.0f32 <= self.internal_offset);
        debug_assert!(self.internal_offset <= 0.9f32);
        self.layer.into_global_offset() + self.internal_offset
    }
}
    
pub struct LayerVisibilityFlags {
    layer_flags : Vec<bool>,
}

impl Inspectable for LayerVisibilityFlags {
    type Attributes = ();

    fn ui(
        &mut self,
        ui : &mut Ui,
        _options : Self::Attributes,
        _context : &mut Context,
    ) -> bool {
        ui.vertical(|ui| {
            Layer::iter()
            .map(|x| x.into())
            .zip(self.layer_flags.iter_mut())
            .for_each(|(label, flag) : (&'static str, &mut bool)| {
                ui.checkbox(flag, label);
            })
        }); false
    }
}

// FIXME it's constantly setting the value.
// That's "stoopid".
pub fn layer_system(
    layer_flags : Res<LayerVisibilityFlags>,
    mut query : Query<(&mut Transform, &LayerComponent)>, 
) {
    for (mut transform, layer) in query.iter_mut() {
        if layer_flags.layer_flags[layer.layer as usize] {
            transform.translation.z = layer.into_global_offset();
        } else {
            transform.translation.z = -1.0f32;
        }
    }
}

pub struct LayerPlugin;

impl Plugin for LayerPlugin {
    fn build(&self, app : &mut App) {
        app
        .insert_resource(LayerVisibilityFlags {
            layer_flags : Layer::iter().map(|_| true).collect(),
        })
        .add_system(layer_system);
    }
}
