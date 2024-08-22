# trve_bevy_font

An opinionated Bevy plugin to load Font Assets quickly and easily. Ideal for Game Jams.

## How to use

This plugin is meant to be a convenience tool to load all font assets for your game at startup, ideal for small projects and prototypes.

To use it, add it to your Cargo.toml file like this:

```toml
trve_bevy_font = { git = "https://github.com/mnmaita/trve_bevy_font" }
```

Remember you can also target tags, commits and branches with this method:

```toml
trve_bevy_font = { git = "https://github.com/mnmaita/trve_bevy_font", tag = "v0.4.0" }
```

```toml
trve_bevy_font = { git = "https://github.com/mnmaita/trve_bevy_font", branch = "test" }
```

```toml
trve_bevy_font = { git = "https://github.com/mnmaita/trve_bevy_font", rev = "some-sha" }
```

### Default usage and overriding default behavior

By default, it will load all assets from a "fonts" directory under your "assets" folder. You can override this directory by using the `FontAssetFolder` Resource:

```rs
let mut app = App::new();

// Your plugins go here.
app.add_plugins(TrveFontPlugin);

// You insert this Resource and use the `new` function
// which accepts any parameter that can be turned into an `AssetPath`.
app.insert_resource(FontAssetFolder::new("ttfs"));
```

This will load all assets from `assets/ttfs` by using `AssetServer`'s `load_folder` method.

### Loading a list of assets

Certain platforms, like web, can't use `load_folder` to load assets so this library provides an override via the `FontAssetList` Resource.

This allows you to load a list of assets from the folder specified in the `FontAssetFolder` Resource, within the `assets` directory.

```rust
    // This will attempt to load `assets/fonts/bold.ttf`, `assets/fonts/italic.ttf` and `assets/fonts/thin.ttf`.
    app.insert_resource(FontAssetList::new(
        [
            "bold.ttf",
            "italic.ttf",
            "thin.ttf",
        ]
        .into(),
    ));
```

```rust
    // This will attempt to load `assets/ttfs/bold.ttf`, `assets/ttfs/italic.ttf` and `assets/ttfs/thin.ttf`.
    app.insert_resource(FontAssetFolder::new("ttfs"));
    app.insert_resource(FontAssetList::new(
        [
            "bold.ttf",
            "italic.ttf",
            "thin.ttf",
        ]
        .into(),
    ));
```

If you insert this Resource the plugin will **only** load the assets provided in the list.

## Bevy version compatibility

| trve_bevy_font | bevy |
| -------------- | ---- |
| 0.3 0.4        | 0.14 |
| 0.2            | 0.13 |
| 0.1            | 0.12 |
