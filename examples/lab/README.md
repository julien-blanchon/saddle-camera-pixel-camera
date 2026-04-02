# Pixel Camera Lab

Crate-local verification app for [`saddle-camera-pixel-camera`](../..). It keeps the shared crate runnable, BRP-friendly, and E2E-testable without relying on project sandboxes.

## Run

```bash
cargo run -p saddle-camera-pixel-camera-lab
```

Set `PIXEL_CAMERA_AUTO_EXIT_SECONDS=3` for batch runs that should exit automatically.

## E2E

```bash
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_smoke
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_subpixel_scroll
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_resize
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_zoom
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_mixed_layers
cargo run -p saddle-camera-pixel-camera-lab --features e2e -- pixel_camera_cursor
```

## BRP

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-camera-pixel-camera-lab
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_camera_pixel_camera_lab.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

The lab scene includes:

- a scrolling checkerboard world on the pixel layer
- a moving actor that makes subpixel drift obvious
- an HD badge and UI overlay on the high-resolution layer
- live `PixelViewportMetrics` readout
- cursor-to-world mapping with a visible marker
