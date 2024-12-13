use bevy::{asset::LoadedFolder, image::ImageSampler, prelude::*};

#[derive(Clone)]
pub struct AtlasOptions {
    pub folder_path: String,
    pub padding: Option<UVec2>,
    pub sampler: Option<ImageSampler>,
}

impl AtlasOptions {
    pub fn new(
        folder_path: String,
        padding: Option<UVec2>,
        sampler: Option<ImageSampler>,
    ) -> AtlasOptions {
        AtlasOptions {
            folder_path,
            padding,
            sampler,
        }
    }
}

#[derive(Resource)]
pub struct LoadedFolders(Vec<Handle<LoadedFolder>>);

pub struct TextureAtlasData {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub atlas_texture: Handle<Image>,
    pub source_data: TextureAtlasSources,
}

#[derive(Resource)]
pub struct TextureAtlases(pub Vec<TextureAtlasData>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AtlasLoadingState {
    #[default]
    Setup,
    Completed,
}

pub struct TextureAtlasPlugin {
    atlas_options: Vec<AtlasOptions>,
}

impl TextureAtlasPlugin {
    pub fn new(atlas_options: Vec<AtlasOptions>) -> Self {
        Self { atlas_options }
    }
}

impl Plugin for TextureAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AtlasLoadingState>()
            .insert_resource(LoadedFolders(Vec::new()))
            .add_systems(OnEnter(AtlasLoadingState::Setup), {
                let atlas_options = self.atlas_options.clone();
                move |commands: Commands, asset_server: Res<AssetServer>| {
                    load_folders(commands, asset_server, &atlas_options)
                }
            })
            .add_systems(
                Update,
                check_folder_loading.run_if(in_state(AtlasLoadingState::Setup)),
            )
            .add_systems(OnExit(AtlasLoadingState::Setup), {
                let atlas_options = self.atlas_options.clone();
                move |commands: Commands,
                      loaded_folders: Res<LoadedFolders>,
                      textures: ResMut<Assets<Image>>,
                      folders_assets: Res<Assets<LoadedFolder>>,
                      atlas_layouts: ResMut<Assets<TextureAtlasLayout>>| {
                    assemble_texture_atlases(
                        commands,
                        &loaded_folders,
                        &atlas_options,
                        folders_assets,
                        textures,
                        atlas_layouts,
                    )
                }
            });
    }
}

fn load_folders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_options: &[AtlasOptions],
) {
    let folder_handles: Vec<_> = atlas_options
        .iter()
        .map(|option| {
            info!("Loading folder: {}", option.folder_path);
            asset_server.load_folder(&option.folder_path)
        })
        .collect();

    commands.insert_resource(LoadedFolders(folder_handles));
}

fn check_folder_loading(
    mut next_state: ResMut<NextState<AtlasLoadingState>>,
    loaded_folders: Res<LoadedFolders>,
    mut asset_events: EventReader<AssetEvent<LoadedFolder>>,
) {
    let all_folders_loaded = loaded_folders.0.iter().all(|folder_handle| {
        asset_events
            .read()
            .any(|event| event.is_loaded_with_dependencies(folder_handle))
    });

    if all_folders_loaded {
        info!("All folders successfully loaded.");
        next_state.set(AtlasLoadingState::Completed);
    }
}

fn assemble_texture_atlases(
    mut commands: Commands,
    loaded_folders: &LoadedFolders,
    atlas_options: &[AtlasOptions],
    folder_assets: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let atlases: Vec<_> = loaded_folders
        .0
        .iter()
        .zip(atlas_options.iter())
        .filter_map(|(folder_handle, options)| {
            let folder = folder_assets.get(folder_handle)?;
            match create_texture_atlas(
                folder,
                options.padding,
                options.sampler.clone(),
                &mut textures,
            ) {
                Some((layout, sources, texture)) => {
                    let layout_handle = atlas_layouts.add(layout);
                    Some(TextureAtlasData {
                        atlas_layout: layout_handle,
                        atlas_texture: texture,
                        source_data: sources,
                    })
                }
                None => {
                    warn!(
                        "Failed to create texture atlas for folder: {}",
                        options.folder_path
                    );
                    None
                }
            }
        })
        .collect();

    info!("Successfully assembled {} texture atlases.", atlases.len());
    commands.insert_resource(TextureAtlases(atlases));
}

fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampler: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> Option<(TextureAtlasLayout, TextureAtlasSources, Handle<Image>)> {
    let mut atlas_builder = TextureAtlasBuilder::default();
    atlas_builder.padding(padding.unwrap_or_default());

    for handle in &folder.handles {
        let texture_id = handle.id().typed_unchecked::<Image>();
        if let Some(texture) = textures.get(texture_id) {
            atlas_builder.add_texture(Some(texture_id), texture);
        } else {
            warn!("Texture not found for handle: {:?}", texture_id);
        }
    }

    atlas_builder
        .build()
        .map(|(layout, sources, atlas_texture)| {
            let texture_handle = textures.add(atlas_texture);

            if let Some(image) = textures.get_mut(&texture_handle) {
                image.sampler = sampler.unwrap_or_default();
            }

            (layout, sources, texture_handle)
        })
        .map_err(|e| {
            error!("Error building texture atlas: {:?}", e);
            e
        })
        .ok()
}
