use bevy::{asset::LoadedFolder, image::ImageSampler, prelude::*};

const SPRITES_FOLDER_PATH: &str = "sprites/ui";

#[derive(Resource, Default)]
struct SpriteFolder(Handle<LoadedFolder>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

#[derive(Component)]
struct Cursor;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), load_textures)
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::Finished), setup)
        .add_systems(Update, draw_cursor)
        .run();
}

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpriteFolder(asset_server.load_folder(SPRITES_FOLDER_PATH)));
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_folder: Res<SpriteFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&sprite_folder.0) {
            next_state.set(AppState::Finished);
        }
    }
}

fn setup(
    mut commands: Commands,
    sprite_handles: Res<SpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut windows: Query<&mut Window>,
) {
    let loaded_folder = loaded_folders.get(&sprite_handles.0).unwrap();

    let (texture_atlas_nearest, nearest_sources, nearest_texture) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::nearest()),
        &mut textures,
    );

    let atlas_nearest_handle = texture_atlases.add(texture_atlas_nearest);

    let vendor_handle: Handle<Image> = asset_server.get_handle("sprites/ui/ui_0028.png").unwrap();

    commands.spawn(Camera2d::default());

    commands.spawn((
        Cursor,
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::splat(2.0),
            ..default()
        },
        Sprite::from_atlas_image(
            nearest_texture.clone(),
            nearest_sources
                .handle(atlas_nearest_handle, &vendor_handle)
                .unwrap(),
        ),
    ));

    commands.spawn((
        Sprite::from_image(nearest_texture.clone()),
        Transform {
            translation: Vec3::new(-250.0, -130.0, 0.0),
            scale: Vec3::splat(0.8),
            ..default()
        },
    ));

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.visible = false;
    }
}

fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, TextureAtlasSources, Handle<Image>) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.padding(padding.unwrap_or_default());

    for handle in folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture_atlas_sources, texture) =
        texture_atlas_builder.build().unwrap();
    let texture = textures.add(texture);

    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture_atlas_sources, texture)
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
