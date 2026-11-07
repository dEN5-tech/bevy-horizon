use bevy_ecs::prelude::*;

use crate::components::{OrbitingEntity, Position, Velocity};
use crate::resources::{HorizonClock, HorizonOutboxCommands, HudDrawBuffer, ScreenState};

/// Central plugin configuring the Bevy ECS World and Schedule for Horizon native mods.
#[derive(Clone, Copy, Debug, Default)]
pub struct HorizonPlugin;

impl HorizonPlugin {
    /// Configures the ECS `World` and `Schedule` with Horizon standard resources and core systems.
    pub fn build(world: &mut World, schedule: &mut Schedule) {
        // Initialize core resources
        if !world.contains_resource::<HorizonClock>() {
            world.insert_resource(HorizonClock::default());
        }
        if !world.contains_resource::<ScreenState>() {
            world.insert_resource(ScreenState::default());
        }
        if !world.contains_resource::<HudDrawBuffer>() {
            world.insert_resource(HudDrawBuffer::new());
        }
        if !world.contains_resource::<HorizonOutboxCommands>() {
            world.insert_resource(HorizonOutboxCommands::new());
        }

        // Register default kinematics & simulation systems
        schedule.add_systems((
            orbit_kinematics_system,
            velocity_integration_system,
        ));
    }
}

/// Computes circular orbit and sinusoidal bobbing for all entities with `OrbitingEntity`.
pub fn orbit_kinematics_system(
    clock: Res<HorizonClock>,
    mut query: Query<(&mut OrbitingEntity, &mut Position)>,
) {
    let delta = clock.delta_secs;
    let time = clock.time_secs;
    for (mut orbit, mut pos) in &mut query {
        orbit.advance(delta);
        let target = orbit.calculate_position(time);
        pos.x = target[0];
        pos.y = target[1];
        pos.z = target[2];
    }
}

/// Integrates velocity into position for physical non-orbiting entities.
pub fn velocity_integration_system(
    clock: Res<HorizonClock>,
    mut query: Query<(&Velocity, &mut Position), Without<OrbitingEntity>>,
) {
    let delta = clock.delta_secs;
    if delta <= 0.0 {
        return;
    }
    for (vel, mut pos) in &mut query {
        pos.x += vel.vx * delta;
        pos.y += vel.vy * delta;
        pos.z += vel.vz * delta;
    }
}
