pub mod rapier_debug {
    use bevy::prelude::*;
    use bevy::sprite::Mesh2dHandle;
    use bevy_rapier2d::prelude::*;
    use bevy_inspector_egui::Inspectable;

    #[derive(Inspectable)]
    pub struct RapierDebugConfig {
        pub display_colliders : bool,
    }

    //pub struct SavedSpriteState

    pub fn debug_routine(
        mut commands : Commands,
        cfg : Res<RapierDebugConfig>,
        query_yes : Query<Entity, (Without<ColliderDebugRender>, With<ColliderTypeComponent>)>,
        query_no : Query<Entity, (With<ColliderDebugRender>, With<ColliderTypeComponent>, With<Mesh2dHandle>)>,
    ) {
        if cfg.display_colliders {
            for entity in query_yes.iter() {
                commands.entity(entity)
                .insert(ColliderDebugRender::default());
            }
        } else {    
            for entity in query_no.iter() {
                commands.entity(entity)
                .remove::<ColliderDebugRender>()
                .remove::<Mesh2dHandle>();
            }
        }
    }

    impl Default for RapierDebugConfig {
        fn default() -> Self {
            RapierDebugConfig {
                display_colliders : false,
            }
        }
    }

    pub struct DebuggerPlugin;

    impl Plugin for DebuggerPlugin {
        fn build(&self, app : &mut App) {
            app
            .add_system(debug_routine)
            .insert_resource(RapierDebugConfig {
                display_colliders : false,
            })
            .add_plugin(RapierRenderPlugin);
        }
    }
}

pub mod layer_debug {
    use bevy::prelude::*;
    pub use crate::layer_system::LayerVisibilityFlags;

    pub struct DebuggerPlugin;

    impl Plugin for DebuggerPlugin {
        fn build(&self, app : &mut App) {
            
        }
    }
}

use bevy::prelude::{ Plugin, App };
use bevy_inspector_egui::{ InspectorPlugin, widgets::ResourceInspector, Inspectable };

#[derive(Inspectable, Default)]
pub struct DebuggerConfigs {
    rapier_debug : ResourceInspector<rapier_debug::RapierDebugConfig>,
    layer_debug : ResourceInspector<layer_debug::LayerVisibilityFlags>,
}

pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app : &mut App) {
        app
        .add_plugin(rapier_debug::DebuggerPlugin)
        .add_plugin(layer_debug::DebuggerPlugin)
        .add_plugin(InspectorPlugin::<DebuggerConfigs>::new());
    }
}
