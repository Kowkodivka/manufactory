use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AtlasLoading {
    #[default]
    Setup,
    Finished,
}

pub struct AtlasPlugin {
    folders: Box<Vec<String>>,
}

impl AtlasPlugin {
    pub fn new(folders: Vec<String>) -> AtlasPlugin {
        AtlasPlugin {
            folders: Box::new(folders),
        }
    }
}

impl Plugin for AtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AtlasLoading>()
            .add_systems(OnEnter(AtlasLoading::Setup), load_textures)
            .add_systems(Update, check_textures.run_if(in_state(AtlasLoading::Setup)));
    }
}

fn load_textures() {}

fn check_textures() {}