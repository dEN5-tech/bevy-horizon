use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// Unique identifier for an entity within the Minecraft world instance.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MinecraftEntityId(pub u32);

impl MinecraftEntityId {
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl From<u32> for MinecraftEntityId {
    #[inline]
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<MinecraftEntityId> for u32 {
    #[inline]
    fn from(val: MinecraftEntityId) -> Self {
        val.0
    }
}

impl Deref for MinecraftEntityId {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for MinecraftEntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Marker component denoting the local client player.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalPlayer;

/// 3D floating-point position in world coordinates.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn as_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub const fn from_array(arr: [f64; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }

    #[inline]
    pub fn distance_sq(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    #[inline]
    pub fn distance_to(&self, other: &Self) -> f64 {
        self.distance_sq(other).sqrt()
    }
}

impl From<[f64; 3]> for Position {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

impl From<Position> for [f64; 3] {
    #[inline]
    fn from(pos: Position) -> Self {
        pos.as_array()
    }
}

/// Yaw and pitch rotation in degrees.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    #[inline]
    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }

    /// Computes the unit gaze forward vector based on Minecraft yaw/pitch angles.
    pub fn forward_vector(&self) -> [f64; 3] {
        let yaw_rad = (self.yaw as f64).to_radians();
        let pitch_rad = (self.pitch as f64).to_radians();
        [
            -yaw_rad.sin() * pitch_rad.cos(),
            -pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        ]
    }
}

/// 3D kinematic velocity.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Velocity {
    #[inline]
    pub const fn new(vx: f64, vy: f64, vz: f64) -> Self {
        Self { vx, vy, vz }
    }

    #[inline]
    pub const fn as_array(&self) -> [f64; 3] {
        [self.vx, self.vy, self.vz]
    }

    #[inline]
    pub const fn from_array(arr: [f64; 3]) -> Self {
        Self {
            vx: arr[0],
            vy: arr[1],
            vz: arr[2],
        }
    }

    #[inline]
    pub fn length(&self) -> f64 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> [f64; 3] {
        let len = self.length();
        if len > 1e-6 {
            [self.vx / len, self.vy / len, self.vz / len]
        } else {
            [0.0, 0.0, 0.0]
        }
    }
}

impl From<[f64; 3]> for Velocity {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

/// Current and maximum entity health.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    #[inline]
    pub const fn new(current: f32, max: f32) -> Self {
        Self { current, max }
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    #[inline]
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    #[inline]
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    #[inline]
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}

/// Minecraft entity registry type or species name (e.g. "minecraft:zombie").
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityKind(pub String);

impl EntityKind {
    #[inline]
    pub fn new(kind: impl Into<String>) -> Self {
        Self(kind.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EntityKind {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for EntityKind {
    #[inline]
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for EntityKind {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Anchor block placed in the world acting as a pedestal.
#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PedestalBlock {
    pub block_type: String,
    pub pos: [i32; 3],
}

impl PedestalBlock {
    #[inline]
    pub fn new(block_type: impl Into<String>, pos: [i32; 3]) -> Self {
        Self {
            block_type: block_type.into(),
            pos,
        }
    }

    /// Center world coordinates directly above the top face of the pedestal block.
    #[inline]
    pub fn anchor_center(&self) -> [f64; 3] {
        [
            self.pos[0] as f64 + 0.5,
            self.pos[1] as f64 + 1.0,
            self.pos[2] as f64 + 0.5,
        ]
    }
}

/// Orbit kinematic component specifying circular movement and sinusoidal bobbing.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrbitingEntity {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub radius: f64,
    pub speed: f64,
    pub angle: f64,
    pub bob_amplitude: f64,
    pub bob_frequency: f64,
}

impl Default for OrbitingEntity {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            center_z: 0.0,
            radius: 3.0,
            speed: 1.0,
            angle: 0.0,
            bob_amplitude: 0.5,
            bob_frequency: 2.0,
        }
    }
}

impl OrbitingEntity {
    pub fn new(center: [f64; 3], radius: f64, speed: f64) -> Self {
        Self {
            center_x: center[0],
            center_y: center[1],
            center_z: center[2],
            radius,
            speed,
            angle: 0.0,
            bob_amplitude: 0.5,
            bob_frequency: 2.0,
        }
    }

    /// Evaluates current orbit position given absolute time in seconds.
    pub fn calculate_position(&self, time_secs: f64) -> [f64; 3] {
        let x = self.center_x + self.radius * self.angle.cos();
        let y = self.center_y + self.bob_amplitude * (self.bob_frequency * time_secs).sin();
        let z = self.center_z + self.radius * self.angle.sin();
        [x, y, z]
    }

    /// Advances the orbit angle by `delta_secs`.
    #[inline]
    pub fn advance(&mut self, delta_secs: f64) {
        self.angle += self.speed * delta_secs;
        if self.angle > std::f64::consts::TAU {
            self.angle -= std::f64::consts::TAU;
        }
    }
}

// Mob Archetype Markers
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zombie;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skeleton;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Creeper;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveAnimal;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostileMob;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Undead;

// Archetype Bundles
#[derive(Bundle, Clone, Debug)]
pub struct LocalPlayerBundle {
    pub marker: LocalPlayer,
    pub id: MinecraftEntityId,
    pub position: Position,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub health: Health,
}

impl LocalPlayerBundle {
    pub fn new(id: u32, pos: [f64; 3], rot: [f32; 2], health: f32, max_health: f32) -> Self {
        Self {
            marker: LocalPlayer,
            id: MinecraftEntityId(id),
            position: Position::from_array(pos),
            rotation: Rotation::new(rot[0], rot[1]),
            velocity: Velocity::default(),
            health: Health::new(health, max_health),
        }
    }
}

#[derive(Bundle, Clone, Debug)]
pub struct MinecraftMobBundle {
    pub id: MinecraftEntityId,
    pub kind: EntityKind,
    pub position: Position,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub health: Health,
}

impl MinecraftMobBundle {
    pub fn new(id: u32, kind: impl Into<String>, pos: [f64; 3], health: f32, max_health: f32) -> Self {
        Self {
            id: MinecraftEntityId(id),
            kind: EntityKind::new(kind),
            position: Position::from_array(pos),
            rotation: Rotation::default(),
            velocity: Velocity::default(),
            health: Health::new(health, max_health),
        }
    }
}

#[derive(Bundle, Clone, Debug)]
pub struct PedestalOrbitBundle {
    pub pedestal: PedestalBlock,
    pub orbit: OrbitingEntity,
    pub position: Position,
}

impl PedestalOrbitBundle {
    pub fn new(block_type: impl Into<String>, block_pos: [i32; 3], radius: f64, speed: f64) -> Self {
        let pedestal = PedestalBlock::new(block_type, block_pos);
        let center = pedestal.anchor_center();
        let orbit = OrbitingEntity::new(center, radius, speed);
        let initial_pos = Position::from_array(orbit.calculate_position(0.0));
        Self {
            pedestal,
            orbit,
            position: initial_pos,
        }
    }
}
