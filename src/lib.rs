use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::text::DefaultTextPipeline;
use bevy::text::Text2dBounds;
use bevy::text::Text2dSize;
use bevy::ui::ExtractedUiNode;
use bevy::ui::ExtractedUiNodes;
use bevy::ui::RenderUiSystem;
use bevy::window::WindowId;

#[derive(Clone, Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct UiLabel;

#[derive(Bundle, Default)]
pub struct UiLabelBundle {
    pub label_marker: UiLabel,
    pub text: Text,
    pub text_2d_size: Text2dSize,
    pub text_2d_bounds: Text2dBounds,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub fn extract_label_uinodes(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlas>>>,
    text_pipeline: Extract<Res<DefaultTextPipeline>>,
    windows: Extract<Res<Windows>>,
    uinode_query: Extract<
        Query<(
            Entity,
            &GlobalTransform,
            &Text,
            &Text2dSize,
        ), With<UiLabel>>,
    >,
) {
    let scale_factor = windows.scale_factor(WindowId::primary()) as f32;
    for (entity, global_transform, text, calculated_size) in uinode_query.iter() {
        if let Some(text_layout) = text_pipeline.get_glyphs(&entity) {
            let text_glyphs = &text_layout.glyphs;
            let (width, height) = (calculated_size.size.x, calculated_size.size.y);
            let alignment_offset = match text.alignment.vertical {
                VerticalAlign::Top => Vec3::new(0.0, -height, 0.0),
                VerticalAlign::Center => Vec3::new(0.0, -height * 0.5, 0.0),
                VerticalAlign::Bottom => Vec3::ZERO,
            } + match text.alignment.horizontal {
                HorizontalAlign::Left => Vec3::ZERO,
                HorizontalAlign::Center => Vec3::new(-width * 0.5, 0.0, 0.0),
                HorizontalAlign::Right => Vec3::new(-width, 0.0, 0.0),
            };

            let mut color = Color::WHITE;
            let mut current_section = usize::MAX;
            for text_glyph in text_glyphs {
                if text_glyph.section_index != current_section {
                    color = text.sections[text_glyph.section_index]
                        .style
                        .color
                        .as_rgba_linear();
                    current_section = text_glyph.section_index;
                }
                let atlas = texture_atlases
                    .get(&text_glyph.atlas_info.texture_atlas)
                    .unwrap();
                let texture = atlas.texture.clone_weak();
                let index = text_glyph.atlas_info.glyph_index as usize;
                let rect = atlas.textures[index];
                let atlas_size = Some(atlas.size);
                let extracted_transform = global_transform.compute_matrix()
                    * Mat4::from_scale(Vec3::splat(scale_factor.recip()))
                    * Mat4::from_translation(
                        alignment_offset * scale_factor + text_glyph.position.extend(0.),
                    );

                extracted_uinodes.uinodes.push(ExtractedUiNode {
                    transform: extracted_transform,
                    color,
                    rect,
                    image: texture,
                    atlas_size,
                    clip: None,
                });
            }
        }
    }
}

pub struct UiLabelPlugin;

impl Plugin for UiLabelPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<UiLabel>();

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app.add_system_to_stage(
            RenderStage::Extract,
            extract_label_uinodes.after(RenderUiSystem::ExtractNode),
        );
    }
}