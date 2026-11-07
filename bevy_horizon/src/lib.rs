//! # bevy_horizon
//!
//! Production-grade Bevy ECS adapter crate for Horizon native Minecraft mods.
//!
//! Provides archetypes, components, events, resources, bidirectional host bridges,
//! and determinism-preserving package build tools.

pub mod bridge;
pub mod build_tool;
pub mod components;
pub mod events;
pub mod plugin;
pub mod prelude;
pub mod resources;

pub use prelude::*;
