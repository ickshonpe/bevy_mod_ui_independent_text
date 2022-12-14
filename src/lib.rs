use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::RenderApp;
use bevy::render::RenderStage;
use bevy::text::DefaultTextPipeline;
use bevy::text::FontAtlasSet;
use bevy::text::Text2dBounds;
use bevy::text::Text2dSize;
use bevy::text::scale_value;
use bevy::ui::ExtractedUiNode;
use bevy::ui::ExtractedUiNodes;
use bevy::ui::RenderUiSystem;
use bevy::utils::HashSet;
use bevy::window::ModifiesWindows;
use bevy::window::WindowId;
use bevy::window::WindowScaleFactorChanged;

/// Newtype wrapper for [`Text`]
/// 
/// Required so that the text isn't also extracted by `extract_text2d_sprite`
/// and consequently drawn twice.
#[derive(Clone, Component, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct UiText(pub Text);

impl From<Text> for UiText {
    fn from(text: Text) -> Self {
        Self(text)
    }
}

impl UiText {
    /// Constructs a [`UiText`] with a single section.
    /// 
    /// See [`Text`] for more details
    pub fn from_section(value: impl Into<String>, style: TextStyle) -> Self {
        Self(Text::from_section(value, style))
    }    
 
    /// Constructs a [`UiText`] from a list of sections.
    /// 
    /// See [`Text`] for more details
    pub fn from_sections(sections: impl IntoIterator<Item = TextSection>) -> Self {
        Self(Text::from_sections(sections))
    }

    /// Appends a new text section to the end of the text.
    pub fn push_section(&mut self, value: impl Into<String>, style: TextStyle) {
        self.sections.push(TextSection { value: value.into(), style });
    }
}

/// Bundle of components needed to draw text to the Bevy UI 
/// at any position and depth
#[derive(Bundle, Default)]
pub struct IndependentTextBundle {
    pub text: UiText,
    pub text_2d_size: Text2dSize,
    pub text_2d_bounds: Text2dBounds,   
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_ui_independent_text_layout(
    mut queue: Local<HashSet<Entity>>,
    mut textures: ResMut<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    windows: Res<Windows>,
    mut scale_factor_changed: EventReader<WindowScaleFactorChanged>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut font_atlas_set_storage: ResMut<Assets<FontAtlasSet>>,
    mut text_pipeline: ResMut<DefaultTextPipeline>,
    mut text_query: Query<(
        Entity,
        Changed<UiText>,
        &UiText,
        Option<&Text2dBounds>,
        &mut Text2dSize,
    )>,
) {
    let factor_changed = scale_factor_changed.iter().last().is_some();
    let scale_factor = windows.scale_factor(WindowId::primary());
    for (entity, text_changed, UiText(text), maybe_bounds, mut calculated_size) in &mut text_query {
        if factor_changed || text_changed || queue.remove(&entity) {
            let text_bounds = match maybe_bounds {
                Some(bounds) => Vec2::new(
                    scale_value(bounds.size.x, scale_factor),
                    scale_value(bounds.size.y, scale_factor),
                ),
                None => Vec2::new(f32::MAX, f32::MAX),
            };
            match text_pipeline.queue_text(
                entity,
                &fonts,
                &text.sections,
                scale_factor,
                text.alignment,
                text_bounds,
                &mut *font_atlas_set_storage,
                &mut *texture_atlases,
                &mut *textures,
            ) {
                Err(TextError::NoSuchFont) => {
                    queue.insert(entity);
                }
                Err(e @ TextError::FailedToAddGlyph(_)) => {
                    panic!("Fatal error when processing text: {}.", e);
                }
                Ok(()) => {
                    let text_layout_info = text_pipeline.get_glyphs(&entity).expect(
                        "Failed to get glyphs from the pipeline that have just been computed",
                    );
                    calculated_size.size = Vec2::new(
                        scale_value(text_layout_info.size.x, 1. / scale_factor),
                        scale_value(text_layout_info.size.y, 1. / scale_factor),
                    );
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn extract_text_sprite(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    texture_atlases: Extract<Res<Assets<TextureAtlas>>>,
    text_pipeline: Extract<Res<DefaultTextPipeline>>,
    windows: Extract<Res<Windows>>,
    text_query: Extract<
        Query<(
            Entity,
            &GlobalTransform,
            &UiText,
            &Text2dSize,
            &ComputedVisibility,
        )>
    >,
) {
    let scale_factor = windows.scale_factor(WindowId::primary()) as f32;
    for (entity, global_transform, text, calculated_size, computed_visibility) in text_query.iter() {
        if !computed_visibility.is_visible() {
            continue;
        }
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

pub struct IndependentTextPlugin;

impl Plugin for IndependentTextPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<UiText>()
        .add_system_to_stage(
            CoreStage::PostUpdate,
            update_ui_independent_text_layout.after(ModifiesWindows),
        );
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app.add_system_to_stage(
            RenderStage::Extract,
            extract_text_sprite.after(RenderUiSystem::ExtractNode),
        ); 
    }
}