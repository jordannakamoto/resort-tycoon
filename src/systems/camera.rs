use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
pub struct CameraController {
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            pan_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.3,
            max_zoom: 3.0,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_pan, camera_zoom));
    }
}

fn camera_pan(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &OrthographicProjection, &CameraController), With<Camera>>,
) {
    let Ok((mut transform, projection, controller)) = query.get_single_mut() else {
        return;
    };

    let mut pan_delta = Vec2::ZERO;

    // Keyboard panning (WASD or Arrow Keys)
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        pan_delta.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        pan_delta.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        pan_delta.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        pan_delta.x += 1.0;
    }

    // Apply keyboard pan
    if pan_delta != Vec2::ZERO {
        pan_delta = pan_delta.normalize();
        transform.translation.x += pan_delta.x * controller.pan_speed * time.delta_secs() * projection.scale;
        transform.translation.y += pan_delta.y * controller.pan_speed * time.delta_secs() * projection.scale;
    }

    // Mouse panning (Middle Mouse Button)
    if mouse_button.pressed(MouseButton::Middle) {
        for motion in mouse_motion.read() {
            // Invert Y to match intuitive dragging
            transform.translation.x -= motion.delta.x * projection.scale;
            transform.translation.y += motion.delta.y * projection.scale;
        }
    }
}

fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut OrthographicProjection, &CameraController), With<Camera>>,
) {
    let Ok((mut projection, controller)) = query.get_single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        // Zoom in/out based on scroll direction
        let zoom_delta = -event.y * controller.zoom_speed;
        projection.scale = (projection.scale + zoom_delta).clamp(controller.min_zoom, controller.max_zoom);
    }
}
