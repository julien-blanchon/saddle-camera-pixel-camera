# Configuration

All sizes are expressed in virtual pixels or world units depending on the field. Window fitting is computed from physical pixels first, then converted back into logical units for Bevy UI and cursor helpers.

## `PixelCamera`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `virtual_size` | `UVec2` | `320x180` | each axis `>= 1` | Target low-resolution canvas before upscale |
| `zoom` | `u32` | `1` | `>= 1` | Logical zoom multiplier. Higher values reduce visible world area while preserving integer scaling |
| `scale_mode` | `PixelCameraScaleMode` | `IntegerLetterbox` | enum values below | Controls how the final canvas fits inside the window |
| `outer_camera_order` | `isize` | `0` | any | Render order for the outer camera. The inner camera uses `order - 1` |
| `letterbox_color` | `Color` | `Color::BLACK` | any | Clear color outside the fitted pixel canvas |
| `world_layers` | `RenderLayers` | `PIXEL_LAYERS` | non-overlapping with `high_res_layers` recommended | Layers rendered by the inner pixel camera |
| `high_res_layers` | `RenderLayers` | `HIGH_RES_LAYERS` | non-overlapping with `world_layers` recommended | Layers rendered by the outer camera together with the canvas |

Notes:

- `virtual_size` is sanitized to at least `1x1`.
- `zoom` is sanitized to at least `1`.
- Integer scale is always computed against `virtual_size`, not against the zoomed world-view size. Zoom changes what the inner camera sees, not how the final canvas fits the window.

### `PixelCameraScaleMode`

| Variant | Effect |
| --- | --- |
| `IntegerLetterbox` | Current default behavior: largest integer scale that fully fits inside the window, centered with bars if needed |
| `IntegerCrop` | Chooses the smallest integer scale that fully covers the window, allowing the canvas to extend past the window edges |
| `FractionalFit` | Uses a non-integer presentation scale so the canvas fits the window as tightly as possible while preserving aspect ratio |

### Preset Constructors

`PixelCamera` also exposes convenience constructors for common retro targets:

- `PixelCamera::nes()`
- `PixelCamera::snes()`
- `PixelCamera::gameboy()`
- `PixelCamera::gba()`

## `PixelCameraTransform`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `logical_position` | `Vec2` | `Vec2::ZERO` | Float-precision camera position authored by game code |

Write this field instead of mutating the helper camera transforms directly.

## `PixelShake`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `amplitude` | `f32` | `0.0` | `>= 0.0` | Maximum shake in virtual pixels before rounding |
| `frequency` | `f32` | `12.0` | `> 0.0` recommended | How quickly the deterministic shake pattern changes over time |
| `decay` | `f32` | `12.0` | `>= 0.0` | Per-second amplitude decay |
| `seed` | `u32` | `0` | any | Deterministic variation for tests and replayable effects |

Shake is applied to the canvas presentation path, not to the inner camera transform. The resulting offset is rounded to whole virtual pixels before being converted into screen space.

## `PixelSnap`

`PixelSnap` has no fields. It rounds an entity's `Transform.translation.x` and `.y` to whole numbers every frame.

Use it for:

- sprites that must stay locked to the texel grid
- debug markers
- cursor reticles

Avoid it for:

- intentionally smooth props or particles
- HD overlays that should not jump by whole pixels

## `PixelViewportMetrics`

This component is runtime output, not authored config.

| Field | Meaning |
| --- | --- |
| `integer_scale` | Current integer upscale factor |
| `presentation_scale` | Actual scale used for the final canvas presentation |
| `zoom` | Sanitized runtime zoom |
| `virtual_size` | Sanitized virtual resolution |
| `world_view_size` | World-space size visible through the inner camera after zoom |
| `overscan` | Extra texels rendered around the virtual frame |
| `render_target_size` | Actual off-screen image size including overscan |
| `window_physical_size` | Physical window size in pixels |
| `window_logical_size` | Logical window size after DPI conversion |
| `scale_factor` | Effective window scale factor |
| `viewport_physical_size` | Fitted canvas size in physical pixels |
| `viewport_origin_physical` | Top-left physical origin of the canvas within the window |
| `viewport_logical_size` | Fitted canvas size in logical points |
| `viewport_origin_logical` | Top-left logical origin of the canvas |
| `snapped_position` | Floor-aligned world position used by the inner camera |
| `fractional_offset` | Logical remainder after snapping |
| `texture_sample_offset` | Texel-space crop offset applied to the canvas sprite |
| `shake_offset` | Current pixel-aligned shake in virtual pixels |

## Filtering And Atlas Notes

- Use `ImagePlugin::default_nearest()`. Linear filtering will blur the upscaled canvas.
- Atlas padding still matters. Nearest sampling can still pull in neighboring texels when the source rect is tight against another sprite.
- Prefer extruded atlases or at least 1 px padding around each sprite.
- Odd sprite sizes plus centered anchors can land a sprite between texels even in a correct camera setup. Align transforms intentionally or use `PixelSnap` where appropriate.

## DPI And Resize Behavior

- Integer scale is derived from physical window pixels.
- The displayed canvas size and origin are also cached in logical units for UI and cursor mapping.
- Changing window size, moving across monitors, or changing `scale_factor_override` all trigger a metrics refresh.

## Cursor Helpers

- `screen_to_virtual` / `screen_to_virtual_physical` return `None` when the pointer is inside the letterbox area.
- `screen_to_world` / `cursor_to_world` use the current `PixelCameraTransform.logical_position` and `world_view_size`.
- World mapping assumes the window origin is top-left and world `+Y` points upward.

## Known Visual Limits

- Rotated or non-uniformly scaled sprites can still shimmer.
- Perspective content is outside the intended use case.
- Pixel-perfect guarantees apply to the low-resolution world path. HD layers intentionally opt out so text and overlays remain sharp at native resolution.
