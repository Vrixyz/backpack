use bevy::prelude::*;

pub mod mouse;

pub fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
