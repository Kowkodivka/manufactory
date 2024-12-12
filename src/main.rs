use bevy::{image::ImageSampler, prelude::*};
use plugins::atlas::{AtlasLoadingState, AtlasOptions, TextureAtlasPlugin, TextureAtlases};

mod plugins;

#[derive(Component)]
struct Cursor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TextureAtlasPlugin::new(vec![AtlasOptions {
            folder_path: "sprites/ui".to_string(),
            padding: None,
            sampler: Some(ImageSampler::nearest()),
        }]))
        .add_systems(OnEnter(AtlasLoadingState::Completed), setup)
        .add_systems(Update, draw_cursor)
        .run();
}

fn setup(
    mut commands: Commands,
    texture_atlases: Res<TextureAtlases>,
    asset_server: Res<AssetServer>,
    mut windows: Query<&mut Window>,
) {
    if let Some(atlas_data) = texture_atlases.0.first() {
        commands.spawn(Camera2d::default());

        let vendor_handle: Handle<Image> =
            asset_server.get_handle("sprites/ui/ui_0028.png").unwrap();

        if let Some(source) = atlas_data
            .source_data
            .handle(atlas_data.atlas_layout.clone(), &mut vendor_handle.clone())
        {
            commands.spawn((
                Cursor,
                Transform {
                    translation: Vec3::new(0.0, 0.0, 10.0),
                    scale: Vec3::splat(2.0),
                    ..default()
                },
                Sprite::from_atlas_image(atlas_data.atlas_texture.clone(), source),
            ));
        }
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.visible = false;
    }
}

fn draw_cursor(
    mut cursors: Query<&mut Transform, With<Cursor>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    if let Ok(mut cursor_transform) = cursors.get_single_mut() {
        cursor_transform.translation.x = point.x;
        cursor_transform.translation.y = point.y;
    }
}
