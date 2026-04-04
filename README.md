# Saddle Camera Pixel Camera

Reusable pixel-perfect 2D camera pipeline for Bevy.

The crate renders a low-resolution world into an off-screen image, upscales it at integer multiples, keeps HD overlays separate, and preserves smooth camera motion by splitting logical camera movement from snapped render movement.

It now also exposes explicit presentation scale modes so retro games can choose between strict integer letterboxing, integer crop, or fractional fit-to-window behavior.

## Quick Start

```toml
[dependencies]
saddle-camera-pixel-camera = { git = "https://github.com/julien-blanchon/saddle-camera-pixel-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_pixel_camera::{PixelCamera, PixelCameraPlugin, PixelCameraTransform, PIXEL_LAYERS};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<DemoState>()
        .add_plugins(PixelCameraPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            Update,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Pixel Camera Root"),
        PixelCamera::new(320, 180),
        PixelCameraTransform {
            logical_position: Vec2::ZERO,
        },
    ));

    commands.spawn((
        Name::new("World Sprite"),
        Sprite::from_color(Color::srgb(0.2, 0.7, 0.4), Vec2::splat(16.0)),
        PIXEL_LAYERS.clone(),
    ));
}
```

Use `PixelCameraPlugin::always_on(Update)` for tools, examples, and always-on scenes.

## Required Setup

- Use `ImagePlugin::default_nearest()` for crisp upscaling.
- Put low-resolution world content on `PIXEL_LAYERS` or on the custom `PixelCamera::world_layers`.
- Put UI, text, debug overlays, and other HD content on `HIGH_RES_LAYERS` or on the custom `PixelCamera::high_res_layers`.
- Keep sprite atlases padded or extruded. Nearest filtering does not prevent atlas bleed by itself.
- Watch odd sprite sizes and centered anchors. A 15 px sprite with a centered anchor still lands between texels unless the authored transform is aligned or `PixelSnap` is used deliberately.

## Public API

| Type | Purpose |
| --- | --- |
| `PixelCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `PixelCameraSystems` | Public ordering hooks: `DetectChanges`, `ComputeMetrics`, `ComputeSubpixel`, `ApplyEffects`, `ApplyTransforms`, `RefitCanvas` |
| `PixelCamera` | Top-level config for virtual resolution, zoom, layers, letterbox color, and camera order |
| `PixelCameraScaleMode` | Presentation fit policy: integer letterbox, integer crop, or fractional fit |
| `PixelCameraTransform` | Logical float-precision camera position written by game code |
| `PixelViewportMetrics` | Readable runtime diagnostics: scale, viewport, snapped position, fractional offset, and render-target sizing |
| `PixelCameraInner` / `PixelCameraCanvas` / `PixelCameraOuter` | Helper markers for BRP and runtime inspection |
| `PixelShake` | Pixel-aligned canvas shake effect |
| `PixelSnap` | Marker that rounds selected entity transforms to whole world pixels each frame |
| `PixelScaleChanged` | Message emitted when integer scale changes |
| `PixelCursorHit` | Cursor helper result with virtual and world coordinates |

## Layer Usage

- `PIXEL_LAYERS`: low-resolution world render path
- `HIGH_RES_LAYERS`: outer-camera presentation path for HD overlays

The default `PixelCamera` uses those constants directly, but each camera root can override them with custom `RenderLayers`.

## Runtime Control

- Move the camera by mutating `PixelCameraTransform.logical_position`.
- Change `PixelCamera.virtual_size`, `zoom`, `letterbox_color`, or layers at runtime; the render target and viewport metrics will refresh automatically.
- Change `PixelCamera.scale_mode` at runtime to swap between strict retro letterboxing, crop-to-fill, or fractional fit behavior.
- Read `PixelViewportMetrics` for current integer scale, viewport rectangle, overscan size, snapped camera position, and fractional remainder.
- Use `screen_to_virtual`, `screen_to_world`, or `cursor_to_world` for click/picking helpers that respect letterboxing and DPI.
- If a helper entity is manually removed during debugging, the next update rebuilds the inner/canvas/outer rig from the root `PixelCamera`.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal low-res world plus HD overlay badge | `cargo run -p saddle-camera-pixel-camera-example-basic` |
| `subpixel` | Smooth camera drift with snapped metrics overlay | `cargo run -p saddle-camera-pixel-camera-example-subpixel` |
| `resize` | Runtime resize and scale-factor override updates | `cargo run -p saddle-camera-pixel-camera-example-resize` |
| `shake` | Pixel-aligned canvas shake pulses | `cargo run -p saddle-camera-pixel-camera-example-shake` |
| `mixed_layers` | World on low-res layers, badge/UI on HD layers | `cargo run -p saddle-camera-pixel-camera-example-mixed-layers` |
| `pixel_cursor` | Cursor-to-virtual and cursor-to-world mapping | `cargo run -p saddle-camera-pixel-camera-example-pixel-cursor` |
| `retro_presets` | Switch between NES, SNES, Game Boy, and GBA-style virtual resolutions | `cargo run -p saddle-camera-pixel-camera-example-retro-presets` |

Set `PIXEL_CAMERA_AUTO_EXIT_SECONDS=3` to auto-exit examples during batch verification.

Every example now embeds a `saddle-pane` panel for live edits to virtual resolution, zoom, scale mode, logical camera offset, and shake tuning.

## Workspace Lab

The richer verification app lives inside the crate at `shared/camera/saddle-camera-pixel-camera/examples/lab`:

```bash
cargo run -p saddle-camera-pixel-camera-lab
```

With E2E enabled:

```bash
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_smoke
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_subpixel_scroll
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_resize
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_zoom
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_mixed_layers
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_cursor
```

## Known Limitations

- Fractional fit is presentation-only. The low-resolution world still renders into the same off-screen target; only the final canvas sizing changes.
- The crate does not solve pixel-perfect rotation or arbitrary sprite scaling. Rotated or non-uniformly scaled sprites can still shimmer.
- Perspective content is outside the core contract.
- Camera-follow policy stays outside the crate. Dead zones, spring follow, room locks, and look-ahead belong in consumer code.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
