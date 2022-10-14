use bevy::prelude::*;
use bevy_mod_ui_label::*;

fn setup(
    mut commands: Commands,
    asset_loader: Res<AssetServer>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(UiLabelBundle {
        label: UiLabel(Text {
            sections: vec![TextSection {
                value: "Hello, world".to_string(), 
                style: TextStyle {
                    font: asset_loader.load("Topaz-8.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE
                },
            }],
            alignment: TextAlignment::CENTER,
        }),
        transform: Transform {
            translation: Vec3::new(400., 300., 100.),
            rotation: Quat::from_rotation_z(8f32.recip() * std::f32::consts::PI),
            ..Default::default()
        },
       ..Default::default()
    });  
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            ..Default::default()
        })      
        .add_plugins(DefaultPlugins)
        .add_plugin(UiLabelPlugin)
        .add_startup_system(setup)
        .run();
}
