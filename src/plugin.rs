use bevy::{
    asset::{AssetPath, LoadedFolder, RecursiveDependencyLoadState},
    prelude::*,
};

const FONT_ASSET_FOLDER: &str = "fonts";

pub struct TrveFontPlugin;

impl Plugin for TrveFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontAssetFolder>();
        app.init_resource::<FontLoadState>();
        app.add_systems(Startup, load_fonts);
        app.add_systems(
            Update,
            update_font_assets_load_state.run_if(not(resource_equals(FontLoadState::Loaded))),
        );
    }
}

#[derive(Resource)]
pub struct FontAssetFolder<'a>(AssetPath<'a>);

impl<'a> FontAssetFolder<'a> {
    pub fn new(path: impl Into<AssetPath<'a>>) -> Self {
        Self(path.into())
    }
}

#[derive(Resource, Default, Deref)]
pub struct FontAssetList<'a>(Vec<AssetPath<'a>>);

impl<'a> FontAssetList<'a> {
    pub fn new(path: Vec<impl Into<AssetPath<'a>>>) -> Self {
        Self(
            path.into_iter()
                .map(|path| path.into())
                .collect::<Vec<AssetPath<'a>>>(),
        )
    }
}

impl Default for FontAssetFolder<'_> {
    fn default() -> Self {
        Self(FONT_ASSET_FOLDER.into())
    }
}

#[derive(Default, Resource, PartialEq)]
enum FontLoadState {
    #[default]
    NotLoaded,
    Loading,
    Loaded,
    Failed,
}

impl From<RecursiveDependencyLoadState> for FontLoadState {
    fn from(value: RecursiveDependencyLoadState) -> Self {
        match value {
            RecursiveDependencyLoadState::NotLoaded => Self::NotLoaded,
            RecursiveDependencyLoadState::Loading => Self::Loading,
            RecursiveDependencyLoadState::Loaded => Self::Loaded,
            RecursiveDependencyLoadState::Failed => Self::Failed,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct FontHandles(Vec<Handle<Font>>);

#[derive(Resource, Default, Deref, DerefMut)]
struct FontFolderHandle(Handle<LoadedFolder>);

fn load_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    font_folder: Res<FontAssetFolder<'static>>,
    font_asset_list: Option<Res<FontAssetList<'static>>>,
) {
    if cfg!(not(target_family = "wasm")) && font_asset_list.is_none() {
        commands.insert_resource(FontFolderHandle(
            asset_server.load_folder(font_folder.0.clone()),
        ));
        return;
    }

    if let Some(font_asset_list) = font_asset_list {
        if font_asset_list.is_empty() {
            if cfg!(target_family = "wasm") {
                info!("FontAssetList Resource is empty.");
            }
        } else {
            commands.insert_resource(FontHandles(
                font_asset_list
                    .iter()
                    .map(|path| asset_server.load::<Font>(path))
                    .collect::<Vec<Handle<Font>>>(),
            ));
        }
    } else if cfg!(target_family = "wasm") {
        warn!("FontAssetList Resource does not exist.");
    }
}

fn update_font_assets_load_state(
    mut font_load_state: ResMut<FontLoadState>,
    asset_server: Res<AssetServer>,
    font_handles: Option<Res<FontHandles>>,
    font_folder_handle: Option<Res<FontFolderHandle>>,
    font_asset_list: Option<Res<FontAssetList<'static>>>,
) {
    if font_asset_list.is_some() {
        if let Some(font_handles) = font_handles {
            let all_loaded = font_handles.iter().all(|handle| {
                asset_server.recursive_dependency_load_state(handle.id())
                    == RecursiveDependencyLoadState::Loaded
            });
            *font_load_state = if all_loaded {
                RecursiveDependencyLoadState::Loaded.into()
            } else {
                RecursiveDependencyLoadState::NotLoaded.into()
            }
        }
    } else if let Some(font_folder_handle) = font_folder_handle {
        *font_load_state = asset_server
            .recursive_dependency_load_state(font_folder_handle.clone())
            .into()
    }
}

pub fn font_assets_loaded() -> impl Condition<()> {
    IntoSystem::into_system(resource_equals(FontLoadState::Loaded))
}
