use bevy::prelude::*;

use super::*;

#[test]
fn shake_offsets_are_pixel_aligned() {
    let shake = PixelShake {
        amplitude: 3.2,
        frequency: 8.0,
        decay: 0.0,
        seed: 42,
    };
    let offset = pixel_aligned_shake_offset(&shake, 0.5);
    assert_eq!(offset.x as f32, (offset.x as f32).round());
    assert_eq!(offset.y as f32, (offset.y as f32).round());
}

#[test]
fn zero_amplitude_produces_zero_offset() {
    assert_eq!(
        pixel_aligned_shake_offset(&PixelShake::default(), 1.0),
        IVec2::ZERO
    );
}

#[test]
fn decay_reduces_amplitude() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    let entity = app
        .world_mut()
        .spawn((
            PixelCamera::default(),
            PixelViewportMetrics::default(),
            PixelShake {
                amplitude: 4.0,
                frequency: 10.0,
                decay: 2.0,
                seed: 1,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(std::time::Duration::from_secs_f32(0.5));
    app.add_systems(Update, apply_canvas_shake);
    app.update();

    let shake = app
        .world()
        .get::<PixelShake>(entity)
        .expect("shake component exists");
    assert!(shake.amplitude < 4.0);
}

#[test]
fn pixel_snap_rounds_selected_transforms() {
    let mut app = App::new();
    let entity = app
        .world_mut()
        .spawn((PixelSnap, Transform::from_xyz(3.4, -2.6, 9.0)))
        .id();
    app.add_systems(Update, snap_marked_entities);
    app.update();

    let transform = app
        .world()
        .get::<Transform>(entity)
        .expect("transform exists");
    assert_eq!(transform.translation, Vec3::new(3.0, -3.0, 9.0));
}
