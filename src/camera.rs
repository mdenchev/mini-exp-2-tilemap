use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection}, math::Vec3Swizzles,
};
use bevy_ecs_tilemap::MapQuery;
use bevy_rapier2d::{physics::RapierConfiguration, prelude::RigidBodyVelocity, na::Vector2};

use crate::Player;

// A simple camera system for moving and zooming the camera.
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut proj) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        //if keyboard_input.pressed(KeyCode::A) {
        //    direction -= Vec3::new(1.0, 0.0, 0.0);
        //}

        //if keyboard_input.pressed(KeyCode::D) {
        //    direction += Vec3::new(1.0, 0.0, 0.0);
        //}

        //if keyboard_input.pressed(KeyCode::W) {
        //    direction += Vec3::new(0.0, 1.0, 0.0);
        //}

        //if keyboard_input.pressed(KeyCode::S) {
        //    direction -= Vec3::new(0.0, 1.0, 0.0);
        //}

        if keyboard_input.pressed(KeyCode::Z) {
            proj.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::X) {
            proj.scale -= 0.1;
        }

        if transform.scale.x < 0.1 {
            transform.scale = Vec3::splat(0.1)
        }

        transform.translation += time.delta_seconds() * direction * 500.;
    }
}

pub fn char_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    rapier_parameters: Res<RapierConfiguration>,
    mut player_info: Query<(&Player, &mut RigidBodyVelocity)>,
    mut qs: QuerySet<(
        Query<&mut Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
    map_query: MapQuery,
) {
    let mut player_translation = Vec3::default();
    for (player, mut rb_vels) in player_info.iter_mut() {
        let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
        let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
        let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        let x_axis = -(left as i8) + right as i8;
        let y_axis = -(down as i8) + up as i8;

        let mut move_delta = Vector2::new(x_axis as f32, y_axis as f32);
        if move_delta != Vector2::zeros() {
            // Note that the RapierConfiguration::Scale factor is also used here to transform
            // the move_delta from: 'pixels/second' to 'physics_units/second'
            move_delta /= move_delta.magnitude() * rapier_parameters.scale;
        }

        // Update the velocity on the rigid_body_component,
        // the bevy_rapier plugin will update the Sprite transform.
        rb_vels.linvel = move_delta * player.0;
    }

    for mut transform in qs.q0_mut().iter_mut() {
        let sprite_pos_z =
            map_query.get_zindex_for_pixel_pos(transform.translation.xy().extend(1.0), 0u16, 1u16);
        transform.translation.z = sprite_pos_z;

        player_translation = transform.translation;
    }

    for mut cam_transform in qs.q1_mut().iter_mut() {
        cam_transform.translation = player_translation;
    }

}
