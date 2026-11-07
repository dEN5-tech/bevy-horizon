//! Convenient re-exports for building Horizon mods with Bevy ECS.

pub use bevy_ecs::prelude::*;

pub use crate::bridge::{HorizonBridge, MobSnapshot};
pub use crate::build_tool::{package_hnmod, CompatibilityJson, HorizonNativeManifest, PackageResult};
pub use crate::components::{
    Creeper, EntityKind, Health, HostileMob, LocalPlayer, LocalPlayerBundle, MinecraftEntityId,
    MinecraftMobBundle, OrbitingEntity, PassiveAnimal, PedestalBlock, PedestalOrbitBundle,
    Position, Rotation, Skeleton, Undead, Velocity, Zombie,
};
pub use crate::events::{
    BlockPlaceEvent, BlockSelectEvent, EntityInteractEvent, HorizonKeyEvent, HudDrawEvent,
    PlayerTickEvent,
};
pub use crate::plugin::{
    orbit_kinematics_system, velocity_integration_system, HorizonPlugin,
};
pub use crate::resources::{
    HorizonClock, HorizonOutboxCommands, HudDrawBuffer, HudDrawCommand, OutboxCommand, ScreenState,
};
