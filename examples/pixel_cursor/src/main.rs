use saddle_camera_pixel_camera_example_support as support;

use bevy::{prelude::*, window::PrimaryWindow};
use saddle_camera_pixel_camera::{
    PixelCamera, PixelCameraPlugin, PixelCameraTransform, PixelViewportMetrics, cursor_to_world,
};
use support::{CursorMarker, OverlayText};

#[derive(Resource)]
struct PixelCameraRoot(Entity);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (update_cursor_marker, update_overlay));
    support::maybe_install_auto_exit(&mut app);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    support::spawn_demo_world(&mut commands, &mut images);
    support::spawn_cursor_marker(&mut commands, &mut images);
    support::spawn_overlay(&mut commands, "pixel_cursor.rs");
    let root = support::spawn_pixel_camera_root(&mut commands, PixelCamera::default(), Vec2::ZERO);
    commands.insert_resource(PixelCameraRoot(root));
}

fn update_cursor_marker(
    root: Res<PixelCameraRoot>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_metrics: Query<&PixelViewportMetrics>,
    camera_transform: Query<&PixelCameraTransform>,
    mut cursor: Query<(&mut Transform, &mut Visibility), With<CursorMarker>>,
) {
    let Ok(metrics) = camera_metrics.get(root.0) else {
        return;
    };
    let Ok(transform) = camera_transform.get(root.0) else {
        return;
    };
    let Ok((mut marker_transform, mut visibility)) = cursor.single_mut() else {
        return;
    };

    if let Some(hit) = cursor_to_world(*window, metrics, transform) {
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
    root: Res<PixelCameraRoot>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_metrics: Query<&PixelViewportMetrics>,
    camera_transform: Query<&PixelCameraTransform>,
    mut text: Query<&mut Text, With<OverlayText>>,
) {
    let Ok(metrics) = camera_metrics.get(root.0) else {
        return;
    };
    let Ok(transform) = camera_transform.get(root.0) else {
        return;
    };
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = if let Some(hit) = cursor_to_world(*window, metrics, transform) {
        format!(
            "pixel_cursor.rs\nvirtual={:?}\nworld={:.1}, {:.1}",
            hit.virtual_pixel, hit.world_position.x, hit.world_position.y
        )
    } else {
        "pixel_cursor.rs\nmove the cursor into the canvas".to_string()
    };
}
