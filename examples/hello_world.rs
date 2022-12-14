use bevy::prelude::*;
use bevy_mod_ui_independent_text::*;

fn setup(
    mut commands: Commands,
    asset_loader: Res<AssetServer>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(IndependentTextBundle {
        text: UiText(Text {
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
            rotation: Quat::from_rotation_z(std::f32::consts::PI / 8.),
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
        .add_plugin(IndependentTextPlugin)
        .add_startup_system(setup)
        .run();
}
