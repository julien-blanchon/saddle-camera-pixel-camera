# Architecture

## Dual-Camera Model

Each `PixelCamera` root owns three helper entities:

1. `PixelCameraInner`: a `Camera2d` that renders only the pixel-world layers into an off-screen `Image`
2. `PixelCameraCanvas`: a sprite that displays that image inside the real window
3. `PixelCameraOuter`: a `Camera2d` that renders the canvas plus HD overlays and clears the letterbox area

That split keeps low-resolution world rendering and HD UI/debug content independent.

## Entity Ownership

The public `PixelCamera` component lives on a root entity that also stores:

- `PixelCameraTransform`: the logical float-precision camera position
- `PixelViewportMetrics`: cached runtime diagnostics
- internal helper references for the inner camera, canvas sprite, outer camera, and render target image handle

If the root loses `PixelCamera`, the runtime removes the helper entities on the next update. If the deactivate schedule runs, the helpers are removed immediately.
If a helper entity is manually despawned during tooling or debugging, the runtime rebuilds the whole helper rig on the next update so the root never keeps stale entity references.

## Data Flow

1. Game code writes `PixelCameraTransform.logical_position`.
2. `ComputeMetrics` refreshes integer scale, viewport fit, logical/physical window sizes, overscan size, and render-target size when config or window layout changes.
3. `ComputeSubpixel` splits logical position into:
   - `snapped_position`: floor-aligned world-space pixel coordinate for the inner camera
   - `fractional_offset`: remaining subpixel delta
4. `ApplyEffects` updates shake and optional `PixelSnap` markers.
5. `ApplyTransforms` writes the snapped transform to the inner camera and keeps the canvas centered inside the fitted viewport.
6. `RefitCanvas` shifts the sampled sprite rect inside the oversized render target so the fractional remainder is presented without exposing undefined edges.

## Why Logical Position Is Separate

Mutating the real camera transform directly forces a choice between:

- smooth motion with blurry sampling
- snapped motion with visible jitter

`PixelCameraTransform` keeps logical camera movement at float precision. The runtime then snaps only the render camera while applying the fractional remainder to the sampled canvas region. That preserves crisp texels and smooth apparent motion at the same time.

## Overscan And Crop

The render target is intentionally oversized by `zoom` pixels on each edge.

Without that overscan, shifting the sampled canvas to account for fractional movement would reveal missing texels along the border. The crate instead renders a slightly larger world view, then crops back down to the requested virtual resolution through `Sprite::rect`.

## Window Metrics

The crate computes integer scale from physical window pixels, not logical points. This matters on HiDPI displays and when dragging a window between monitors with different scale factors.

Cached metrics include:

- physical window and viewport size
- logical window and viewport size
- integer scale
- virtual resolution
- zoom-adjusted world view size
- snapped position and fractional remainder
- texture sample offset and current shake offset

## System Ordering

The public `PixelCameraSystems` enum reflects the intended frame pipeline:

1. `DetectChanges`
2. `ComputeMetrics`
3. `ComputeSubpixel`
4. `ApplyEffects`
5. `ApplyTransforms`
6. `RefitCanvas`

Consumers can order their own systems around those phases. Typical integrations place camera-follow logic before `ComputeSubpixel` and late presentation/debug work after `ApplyTransforms`.

## Mixed Content

The intended mixed-content path is:

- pixel art sprites, tilemaps, and low-res props on `world_layers`
- UI, text, debug overlays, and HD presentation helpers on `high_res_layers`

The outer camera sees both the canvas sprite and the HD layers, so UI stays sharp even while the world is rendered at a much lower resolution.

## Future Extension Points

- alternate upscale materials or post-filters layered on the canvas path
- multiple independent pixel-camera rigs for split-screen or editor panels
- transparent-canvas presentation
- optional follow-helper modules that stay separate from the core render path
