#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_camera_pixel_camera_example_support as support;

use bevy::prelude::*;
#[cfg(feature = "brp")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_camera_pixel_camera::{
    PixelCamera, PixelCameraPlugin, PixelCameraTransform, PixelScaleChanged, PixelShake,
    PixelViewportMetrics, cursor_to_world,
};
use support::{CursorMarker, DemoActor, HighResBadge, OverlayText};

#[derive(Resource, Clone, Copy)]
pub struct LabRoot(pub Entity);

#[derive(Resource, Clone, Debug)]
pub struct LabDiagnostics {
    pub scale_events: u32,
    pub last_scale: Option<(u32, u32)>,
    pub last_cursor_hit: Option<saddle_camera_pixel_camera::PixelCursorHit>,
}

impl Default for LabDiagnostics {
    fn default() -> Self {
        Self {
            scale_events: 0,
            last_scale: None,
            last_cursor_hit: None,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct LabCameraMotion {
    pub enabled: bool,
    pub velocity: Vec2,
}

impl Default for LabCameraMotion {
    fn default() -> Self {
        Self {
            enabled: false,
            velocity: Vec2::ZERO,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pixel Camera Lab".into(),
                    resolution: (1440, 900).into(),
                    ..default()
                }),
                ..default()
            }),
    );
    app.add_plugins(PixelCameraPlugin::default());
    #[cfg(feature = "brp")]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default()),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::PixelCameraLabE2EPlugin);

    app.insert_resource(LabDiagnostics::default());
    app.insert_resource(LabCameraMotion::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_actor,
            animate_badge,
            drive_camera_motion,
            track_scale_changes,
            update_cursor_marker,
            update_overlay,
        ),
    );
    support::install_pane(&mut app);
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let camera = PixelCamera {
        virtual_size: support::DEFAULT_VIRTUAL_SIZE,
        zoom: 1,
        ..default()
    };
    let transform = PixelCameraTransform {
        logical_position: Vec2::new(-12.5, 8.0),
    };
    let shake = PixelShake {
        frequency: 16.0,
        decay: 12.0,
        ..default()
    };
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_cursor_marker(&mut commands, &mut images);
    support::spawn_overlay(&mut commands, "pixel_camera_lab");
    let root = commands
        .spawn((
            Name::new("Lab Pixel Camera"),
            camera.clone(),
            transform.clone(),
            shake,
        ))
        .id();
    commands.insert_resource(LabRoot(root));
    support::queue_example_pane(
        &mut commands,
        support::ExamplePixelPane::from_setup(
            &camera,
            transform.logical_position,
            Some(&PixelShake {
                frequency: 16.0,
                decay: 12.0,
                ..default()
            }),
        ),
    );
}

fn animate_actor(time: Res<Time>, mut actors: Query<&mut Transform, With<DemoActor>>) {
    for mut transform in &mut actors {
        let t = time.elapsed_secs();
        transform.translation.x = (t * 1.1).sin() * 58.0;
        transform.translation.y = (t * 2.0).cos() * 22.0;
    }
}

fn animate_badge(time: Res<Time>, mut badges: Query<&mut Transform, With<HighResBadge>>) {
    for mut transform in &mut badges {
        let t = time.elapsed_secs();
        transform.translation.x = 282.0 + (t * 1.0).cos() * 24.0;
        transform.translation.y = 126.0 + (t * 1.2).sin() * 18.0;
        transform.rotation = Quat::from_rotation_z((t * 1.1).sin() * 0.16);
    }
}

fn drive_camera_motion(
    time: Res<Time>,
    motion: Res<LabCameraMotion>,
    root: Res<LabRoot>,
    mut transforms: Query<&mut PixelCameraTransform>,
) {
    if !motion.enabled {
        return;
    }

    let Ok(mut transform) = transforms.get_mut(root.0) else {
        return;
    };
    transform.logical_position += motion.velocity * time.delta_secs();
}

fn track_scale_changes(
    mut diagnostics: ResMut<LabDiagnostics>,
    mut scale_changes: MessageReader<PixelScaleChanged>,
) {
    for event in scale_changes.read() {
        diagnostics.scale_events += 1;
        diagnostics.last_scale = Some((event.old_scale, event.new_scale));
    }
}

fn update_cursor_marker(
    root: Res<LabRoot>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    metrics: Query<&PixelViewportMetrics>,
    camera_transform: Query<&PixelCameraTransform>,
    mut diagnostics: ResMut<LabDiagnostics>,
    mut cursor_marker: Query<(&mut Transform, &mut Visibility), With<CursorMarker>>,
) {
    let Ok(metrics) = metrics.get(root.0) else {
        return;
    };
    let Ok(camera_transform) = camera_transform.get(root.0) else {
        return;
    };
    let Ok((mut marker_transform, mut visibility)) = cursor_marker.single_mut() else {
        return;
    };

    diagnostics.last_cursor_hit = cursor_to_world(*window, metrics, camera_transform);
    if let Some(hit) = diagnostics.last_cursor_hit.as_ref() {
        marker_transform.translation = Vec3::new(
            hit.world_position.x.round(),
            hit.world_position.y.round(),
            4.0,
        );
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn update_overlay(
    root: Res<LabRoot>,
    diagnostics: Res<LabDiagnostics>,
    metrics: Query<&PixelViewportMetrics>,
    mut text: Query<&mut Text, With<OverlayText>>,
) {
    let Ok(metrics) = metrics.get(root.0) else {
        return;
    };
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    let cursor_line = diagnostics
        .last_cursor_hit
        .as_ref()
        .map(|hit| {
            format!(
                "cursor world: {:.1}, {:.1}",
                hit.world_position.x, hit.world_position.y
            )
        })
        .unwrap_or_else(|| "cursor world: outside canvas".to_string());

    text.0 = format!(
        "pixel_camera_lab\nscale={} zoom={}\nsnapped={:?}\nfractional={:.2}, {:.2}\n{}\nscale events={}",
        metrics.integer_scale,
        metrics.zoom,
        metrics.snapped_position,
        metrics.fractional_offset.x,
        metrics.fractional_offset.y,
        cursor_line,
        diagnostics.scale_events,
    );
}
