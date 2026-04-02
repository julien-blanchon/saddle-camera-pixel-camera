use bevy::prelude::*;

use super::*;

#[test]
fn window_change_messages_increment_layout_version() {
    let mut app = App::new();
    app.init_resource::<PixelCameraWindowLayoutVersion>();
    app.add_message::<WindowResized>();
    app.add_message::<WindowScaleFactorChanged>();
    app.add_systems(Update, track_window_changes);

    app.world_mut().write_message(WindowResized {
        window: Entity::PLACEHOLDER,
        width: 640.0,
        height: 360.0,
    });
    app.update();

    assert_eq!(
        app.world().resource::<PixelCameraWindowLayoutVersion>().0,
        1
    );
}
