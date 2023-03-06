use bevy::{prelude::*, render::camera::RenderTarget, window::PrimaryWindow};

/// Used to help identify our main camera
#[derive(Component)]
pub struct GameCamera;

#[derive(Resource, Default)]
pub struct MousePos(pub Vec2);

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePos>();
        app.add_system(my_cursor_system);
    }
}

pub fn my_cursor_system(
    // need to get window dimensions
    primary_query: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut mouse_res: ResMut<MousePos>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let Ok(wnd) = primary_query.get_single() else {
        return;
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        mouse_res.0 = world_pos;
    }
}
