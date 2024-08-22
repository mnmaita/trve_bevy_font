use bevy::{
    asset::{AssetPath, LoadedFolder, RecursiveDependencyLoadState},
    prelude::*,
};

const FONT_ASSET_FOLDER: &str = "fonts";

pub struct TrveFontPlugin;

impl Plugin for TrveFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FontLoadState>();
        app.add_systems(
            Startup,
            (setup_resources, load_fonts.after(setup_resources)),
        );
        app.add_systems(
            Update,
            update_font_assets_load_state.run_if(not(resource_equals(FontLoadState::LOADED))),
        );
    }
}

/// Determines the name of the directory (within the `assets` directory) from where fonts will be loaded.
///
/// By default, this is set to "fonts".
///
/// Since `AssetServer::load_folder()` is unsupported in web builds, it will only be used as the base
/// directory for the file names in the `FontAssetList` Resource.
#[derive(Resource)]
pub struct FontAssetFolder<'a>(AssetPath<'a>);

impl<'a> FontAssetFolder<'a> {
    pub fn new(path: impl Into<AssetPath<'a>>) -> Self {
        Self(path.into())
    }
}

impl Default for FontAssetFolder<'_> {
    fn default() -> Self {
        Self(FONT_ASSET_FOLDER.into())
    }
}

impl std::fmt::Display for FontAssetFolder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// List of assets to be loaded from the directory specified in the `FontAssetFolder` Resource.
///
/// Should be a list of file names with their extension.
///
/// This works as an override for `FontAssetFolder` in non-web platforms so, if set,
/// assets will be loaded individually and only from this list.
///
/// In web builds this is the default and the only supported option.
///
/// Example:
///
/// ```
/// app.insert_resource(FontAssetList::new(
///     [
///         "font1.ttf",
///         "font2.ttf",
///         "font3.ttf",
///     ]
///     .to_vec(),
/// ));
/// ```
#[derive(Resource, Default, Deref)]
pub struct FontAssetList<'a>(Vec<AssetPath<'a>>);

impl<'a> FontAssetList<'a> {
    pub fn new(paths: Vec<impl Into<AssetPath<'a>>>) -> Self {
        let asset_paths: Vec<AssetPath<'a>> = paths.into_iter().map(|path| path.into()).collect();
        Self(asset_paths)
    }
}

#[derive(Resource, PartialEq, Deref)]
struct FontLoadState(RecursiveDependencyLoadState);

impl Default for FontLoadState {
    fn default() -> Self {
        Self(RecursiveDependencyLoadState::NotLoaded)
    }
}

impl FontLoadState {
    const LOADED: Self = Self(RecursiveDependencyLoadState::Loaded);
}

#[derive(Resource, Default, Deref, DerefMut)]
struct FontHandles(Vec<Handle<Font>>);

#[derive(Resource, Default, Deref, DerefMut)]
struct FontFolderHandle(Handle<LoadedFolder>);

fn setup_resources(mut commands: Commands) {
    commands.init_resource::<FontAssetFolder>();

    if cfg!(target_family = "wasm") {
        commands.init_resource::<FontAssetList>();
    }
}

fn load_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    font_folder: Res<FontAssetFolder<'static>>,
    font_asset_list: Option<Res<FontAssetList<'static>>>,
) {
    if cfg!(not(target_family = "wasm")) {
        if font_asset_list.is_none() {
            // TODO: Verify that files in the directory are actually Font handles
            commands.insert_resource(FontFolderHandle(
                asset_server.load_folder(font_folder.0.clone()),
            ));
            return;
        }
    }

    if let Some(font_asset_list) = font_asset_list {
        let load_font_asset = |path| asset_server.load::<Font>(format!("{}/{path}", *font_folder));
        let handles: Vec<Handle<Font>> = match font_asset_list.is_empty() {
            true => Vec::default(),
            false => font_asset_list.iter().map(load_font_asset).collect(),
        };
        commands.insert_resource(FontHandles(handles));
    }
}

fn update_font_assets_load_state(
    mut font_load_state: ResMut<FontLoadState>,
    asset_server: Res<AssetServer>,
    font_handles: Option<Res<FontHandles>>,
    font_folder_handle: Option<Res<FontFolderHandle>>,
    font_asset_list: Option<Res<FontAssetList<'static>>>,
) {
    if cfg!(not(target_family = "wasm")) {
        if font_asset_list.is_none() {
            font_load_state.0 =
                asset_server.recursive_dependency_load_state(&font_folder_handle.unwrap().0);
            return;
        }
    }

    if let Some(font_handles) = font_handles {
        let all_loaded = font_handles.iter().all(|handle| {
            if RecursiveDependencyLoadState::Failed
                == asset_server.recursive_dependency_load_state(handle)
            {
                if let Some(path) = handle.path() {
                    info!("Asset '{path}' failed to load. Make sure the file name is correct and is a font.");
                }
                return true;
            }
            asset_server.is_loaded_with_dependencies(handle)
        });

        font_load_state.0 = match all_loaded {
            true => RecursiveDependencyLoadState::Loaded,
            false => RecursiveDependencyLoadState::NotLoaded,
        };
    }
}

pub fn font_assets_loaded() -> impl Condition<()> {
    IntoSystem::into_system(resource_equals(FontLoadState::LOADED))
}
