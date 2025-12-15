# bevy_step_loader

An [STEP](https://wikipedia.org/wiki/STEP_(file_format)) loader for [bevy](https://bevyengine.org/).

STEP is a widely used CAD data exchange format, that represents 3D solids.

It is supported as an output format by most CAD software.

## Usage

1. Add `bevy_step_loader` to your `Cargo.toml`
2. Add `bevy_step_loader::StepPlugin` plugin to the bevy `App`
3. Load STEP assets by passing paths with ".step"/".stp" extension to `asset_server.load(..)`

### Example

```rs
fn main() {
    App::new()
        .add_plugins(bevy_step_loader::StepPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Mesh3d(asset_server.load("arm.step")),
    ));
}
```
