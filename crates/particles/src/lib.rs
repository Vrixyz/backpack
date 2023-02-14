use bevy::prelude::*;
use rand::Rng;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ParticleExplosion>()
            .add_system(update_velocity)
            .add_system(destroy_after)
            .add_system(handle_particle_events);
    }
}

#[derive(Debug)]
pub struct ParticleExplosion {
    pub location: Vec2,
    pub color: Color,
    pub count: i32,
    pub size: f32,
}

#[derive(Component)]
pub struct DestroyAfter {
    time_to_destroy: f32,
}

impl DestroyAfter {
    pub fn new(time_to_destroy: f32) -> Self {
        Self { time_to_destroy }
    }
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

fn update_velocity(time: Res<Time>, mut q_vel: Query<(&mut Transform, &Velocity)>) {
    for (mut t, vel) in q_vel.iter_mut() {
        t.translation += (vel.0 * time.delta_seconds()).extend(0f32);
    }
}

fn destroy_after(mut commands: Commands, time: Res<Time>, q_des: Query<(Entity, &DestroyAfter)>) {
    for (e, d) in q_des.iter() {
        if d.time_to_destroy <= time.elapsed_seconds() {
            commands.entity(e).despawn();
        }
    }
}

fn handle_particle_events(
    mut commands: Commands,
    time: Res<Time>,
    mut evt_particles: EventReader<ParticleExplosion>,
) {
    for p in evt_particles.iter() {
        let time_to_die = time.elapsed_seconds() + 1f32;
        for _ in 0..p.count {
            let mut offset: Vec2 = rand::thread_rng().gen::<(f32, f32)>().into();
            offset -= Vec2::new(0.5f32, 0.5f32).normalize_or_zero();
            let position = p.location + (offset * 50f32);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: p.color,
                        custom_size: Some(Vec2::splat(p.size)),
                        ..default()
                    },
                    transform: Transform::from_translation(position.extend(1f32)),
                    ..default()
                },
                Velocity(offset * 2000f32),
                DestroyAfter::new(time_to_die),
            ));
        }
    }
}
