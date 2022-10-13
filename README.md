# bevy_mod_ui_label
[![crates.io](https://img.shields.io/crates/v/bevy_mod_ui_label)](https://crates.io/crates/bevy_mod_ui_label)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/ickshonpe/bevy_mod_ui_label)
[![crates.io](https://img.shields.io/crates/d/bevy_mod_ui_label)](https://crates.io/crates/bevy_mod_ui_label)

Draw text anywhere with the Bevy UI.

## Usage

Add the dependency to `Cargo.toml`:

```toml
bevy_mod_ui_label = "0.2.1"
```

Add the plugin to your app:

```rust
use bevy_mod_ui_label::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(UiLabelPlugin)
        // ..rest of app
        .run()
}
```

Don't forget a camera:

```rust
commands.spawn_bundle(Camera2dBundle::default());
```

Then you can spawn a UiLabelBundle:

```rust
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
        transform: Transform::from_translation(Vec3::new(400., 300., 100.)),
       ..Default::default()
    });
```

## Full Example

```
cargo --run --example hello_world
```
