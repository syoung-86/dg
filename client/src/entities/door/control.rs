use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use lib::components::{Open, Door};
pub fn open_door(mut query: Query<&mut Transform, (Added<Open>, With<Door>)>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(FRAC_PI_2);
    }
}
