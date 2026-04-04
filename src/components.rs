use bevy::{camera::visibility::RenderLayers, prelude::*, reflect::Reflect};

pub const PIXEL_LAYER_INDEX: usize = 0;
pub const HIGH_RES_LAYER_INDEX: usize = 1;
pub const PIXEL_LAYERS: RenderLayers = RenderLayers::layer(PIXEL_LAYER_INDEX);
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(HIGH_RES_LAYER_INDEX);

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PixelCameraScaleMode {
    #[default]
    IntegerLetterbox,
    IntegerCrop,
    FractionalFit,
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug, Clone)]
#[require(
    PixelCameraTransform,
    PixelViewportMetrics,
    PixelCameraWindowVersion,
    Transform,
    GlobalTransform,
    Visibility,
    InheritedVisibility
)]
pub struct PixelCamera {
    pub virtual_size: UVec2,
    pub zoom: u32,
    pub scale_mode: PixelCameraScaleMode,
    pub outer_camera_order: isize,
    pub letterbox_color: Color,
    pub world_layers: RenderLayers,
    pub high_res_layers: RenderLayers,
}

impl PixelCamera {
    pub fn new(virtual_width: u32, virtual_height: u32) -> Self {
        Self {
            virtual_size: UVec2::new(virtual_width, virtual_height),
            ..default()
        }
    }

    pub fn nes() -> Self {
        Self::new(256, 240)
    }

    pub fn snes() -> Self {
        Self::new(256, 224)
    }

    pub fn gameboy() -> Self {
        Self::new(160, 144)
    }

    pub fn gba() -> Self {
        Self::new(240, 160)
    }
}

impl Default for PixelCamera {
    fn default() -> Self {
        Self {
            virtual_size: UVec2::new(320, 180),
            zoom: 1,
            scale_mode: PixelCameraScaleMode::IntegerLetterbox,
            outer_camera_order: 0,
            letterbox_color: Color::BLACK,
            world_layers: PIXEL_LAYERS.clone(),
            high_res_layers: HIGH_RES_LAYERS.clone(),
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default, Debug, Clone)]
pub struct PixelCameraTransform {
    pub logical_position: Vec2,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default, Debug, Clone)]
pub struct PixelViewportMetrics {
    pub integer_scale: u32,
    pub presentation_scale: f32,
    pub zoom: u32,
    pub virtual_size: UVec2,
    pub world_view_size: Vec2,
    pub overscan: UVec2,
    pub render_target_size: UVec2,
    pub window_physical_size: UVec2,
    pub window_logical_size: Vec2,
    pub scale_factor: f64,
    pub viewport_physical_size: UVec2,
    pub viewport_origin_physical: IVec2,
    pub viewport_logical_size: Vec2,
    pub viewport_origin_logical: Vec2,
    pub snapped_position: IVec2,
    pub fractional_offset: Vec2,
    pub texture_sample_offset: Vec2,
    pub shake_offset: IVec2,
}

impl PixelViewportMetrics {
    pub fn contains_logical_point(&self, point: Vec2) -> bool {
        point.x >= self.viewport_origin_logical.x
            && point.y >= self.viewport_origin_logical.y
            && point.x < self.viewport_origin_logical.x + self.viewport_logical_size.x
            && point.y < self.viewport_origin_logical.y + self.viewport_logical_size.y
    }

    pub fn contains_physical_point(&self, point: Vec2) -> bool {
        point.x >= self.viewport_origin_physical.x as f32
            && point.y >= self.viewport_origin_physical.y as f32
            && point.x
                < (self.viewport_origin_physical.x + self.viewport_physical_size.x as i32) as f32
            && point.y
                < (self.viewport_origin_physical.y + self.viewport_physical_size.y as i32) as f32
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component, Debug, Clone)]
pub struct PixelCameraCanvas {
    pub root: Entity,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component, Debug, Clone)]
pub struct PixelCameraInner {
    pub root: Entity,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component, Debug, Clone)]
pub struct PixelCameraOuter {
    pub root: Entity,
}

#[derive(Component, Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default, Debug, Clone)]
pub struct PixelSnap;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug, Clone)]
pub struct PixelShake {
    pub amplitude: f32,
    pub frequency: f32,
    pub decay: f32,
    pub seed: u32,
}

impl Default for PixelShake {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            frequency: 12.0,
            decay: 12.0,
            seed: 0,
        }
    }
}

#[derive(Message, Reflect, Clone, Debug, PartialEq, Eq)]
#[reflect(Debug, Clone, PartialEq)]
pub struct PixelScaleChanged {
    pub camera: Entity,
    pub old_scale: u32,
    pub new_scale: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelCursorHit {
    pub virtual_position: Vec2,
    pub virtual_pixel: UVec2,
    pub world_position: Vec2,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct PixelCameraChildren {
    pub inner: Entity,
    pub canvas: Entity,
    pub outer: Entity,
    pub image: Handle<Image>,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct PixelCameraWindowVersion(pub u64);
