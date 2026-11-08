use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::plugin::HorizonPlugin;
use crate::resources::*;

/// A lightweight snapshot of mob data ingested from Minecraft.
#[derive(Clone, Debug, PartialEq)]
pub struct MobSnapshot {
    pub id: u32,
    pub kind: String,
    pub pos: [f64; 3],
    pub health: f32,
    pub max_health: f32,
}

impl MobSnapshot {
    pub fn new(id: u32, kind: impl Into<String>, pos: [f64; 3], health: f32, max_health: f32) -> Self {
        Self {
            id,
            kind: kind.into(),
            pos,
            health,
            max_health,
        }
    }
}

/// Bidirectional bridge coordinating Horizon client runtime states with the Bevy ECS World.
pub struct HorizonBridge {
    pub world: World,
    pub schedule: Schedule,
    player_entity: Option<Entity>,
    mob_map: HashMap<u32, Entity>,
}

impl Default for HorizonBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HorizonBridge {
    /// Creates a newly initialized `HorizonBridge` with registered standard resources and events.
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        HorizonPlugin::build(&mut world, &mut schedule);

        Self {
            world,
            schedule,
            player_entity: None,
            mob_map: HashMap::new(),
        }
    }

    /// Ingests player state from Horizon into ECS `LocalPlayerBundle`.
    pub fn ingest_player(
        &mut self,
        id: u32,
        pos: [f64; 3],
        rot: [f32; 2],
        health: f32,
        max_health: f32,
    ) -> Entity {
        if let Some(entity) = self.player_entity {
            if let Ok(mut entity_mut) = self.world.get_entity_mut(entity) {
                if let Some(mut p) = entity_mut.get_mut::<Position>() {
                    p.x = pos[0];
                    p.y = pos[1];
                    p.z = pos[2];
                }
                if let Some(mut r) = entity_mut.get_mut::<Rotation>() {
                    r.yaw = rot[0];
                    r.pitch = rot[1];
                }
                if let Some(mut h) = entity_mut.get_mut::<Health>() {
                    h.current = health;
                    h.max = max_health;
                }
                return entity;
            }
        }

        let entity = self
            .world
            .spawn(LocalPlayerBundle::new(id, pos, rot, health, max_health))
            .id();
        self.player_entity = Some(entity);
        entity
    }

    /// Ingests or updates an individual mob entity into the ECS world.
    pub fn ingest_mob(
        &mut self,
        id: u32,
        kind: &str,
        pos: [f64; 3],
        health: f32,
        max_health: f32,
    ) -> Entity {
        if let Some(&entity) = self.mob_map.get(&id) {
            if let Ok(mut entity_mut) = self.world.get_entity_mut(entity) {
                if let Some(mut p) = entity_mut.get_mut::<Position>() {
                    p.x = pos[0];
                    p.y = pos[1];
                    p.z = pos[2];
                }
                if let Some(mut h) = entity_mut.get_mut::<Health>() {
                    h.current = health;
                    h.max = max_health;
                }
                return entity;
            }
        }

        // Mob does not exist yet; spawn bundle and attach archetype markers
        let mut entity_mut = self
            .world
            .spawn(MinecraftMobBundle::new(id, kind, pos, health, max_health));

        let kind_lower = kind.to_lowercase();
        if kind_lower.contains("zombie") {
            entity_mut.insert((Zombie, Undead, HostileMob));
        } else if kind_lower.contains("skeleton") {
            entity_mut.insert((Skeleton, Undead, HostileMob));
        } else if kind_lower.contains("creeper") {
            entity_mut.insert((Creeper, HostileMob));
        } else if kind_lower.contains("cow")
            || kind_lower.contains("sheep")
            || kind_lower.contains("pig")
            || kind_lower.contains("chicken")
        {
            entity_mut.insert(PassiveAnimal);
        }

        let entity = entity_mut.id();
        self.mob_map.insert(id, entity);
        entity
    }

    /// Ingests a batch of mobs, keeping tracking IDs in sync.
    pub fn ingest_mobs(&mut self, mobs: &[MobSnapshot]) {
        for mob in mobs {
            self.ingest_mob(mob.id, &mob.kind, mob.pos, mob.health, mob.max_health);
        }
    }

    /// Despawns any ECS mobs that are no longer present in the active live IDs slice.
    pub fn sync_despawned_mobs(&mut self, live_ids: &[u32]) {
        self.mob_map.retain(|id, entity| {
            if live_ids.contains(id) {
                true
            } else {
                let _ = self.world.despawn(*entity);
                false
            }
        });
    }

    /// Triggers a `HorizonKeyEvent` into the ECS event channel.
    pub fn trigger_key_event(&mut self, key: impl Into<String>, pressed: bool) {
        self.world.trigger(HorizonKeyEvent::new(key, pressed));
    }

    /// Triggers an `EntityInteractEvent` into the ECS event channel.
    pub fn trigger_interact_event(
        &mut self,
        entity_id: u32,
        kind: impl Into<String>,
        stack_name: Option<String>,
    ) {
        self.world
            .trigger(EntityInteractEvent::new(entity_id, kind, stack_name));
    }

    /// Triggers a `BlockSelectEvent` into the ECS event channel.
    pub fn trigger_block_select_event(&mut self, block_type: impl Into<String>) {
        self.world.trigger(BlockSelectEvent::new(block_type));
    }

    /// Triggers a `BlockPlaceEvent` into the ECS event channel.
    pub fn trigger_block_place_event(
        &mut self,
        dimension: i32,
        pos: [i32; 3],
        block_type: impl Into<String>,
    ) {
        self.world
            .trigger(BlockPlaceEvent::new(dimension, pos, block_type));
    }

    /// Advances simulation clock, sends `PlayerTickEvent`, and executes the ECS schedule.
    pub fn step_tick(&mut self, delta_secs: f64) {
        let current_tick = if let Some(mut clock) = self.world.get_resource_mut::<HorizonClock>() {
            clock.advance(delta_secs);
            clock.tick
        } else {
            0
        };
        self.world.trigger(PlayerTickEvent::new(current_tick));
        self.schedule.run(&mut self.world);
    }

    /// Sends `HudDrawEvent` and runs the ECS schedule to update HUD elements.
    pub fn step_hud(&mut self, delta_secs: f64) {
        self.world.trigger(HudDrawEvent::new(delta_secs));
        self.schedule.run(&mut self.world);
    }

    /// Drains all commands accumulated in `HorizonOutboxCommands`.
    pub fn drain_outbox(&mut self) -> Vec<OutboxCommand> {
        self.world
            .get_resource_mut::<HorizonOutboxCommands>()
            .map(|mut outbox| outbox.drain_commands())
            .unwrap_or_default()
    }

    /// Drains all draw calls accumulated in `HudDrawBuffer`.
    pub fn drain_hud(&mut self) -> Vec<HudDrawCommand> {
        self.world
            .get_resource_mut::<HudDrawBuffer>()
            .map(|mut hud| hud.drain_commands())
            .unwrap_or_default()
    }

    /// Retrieves the entity ID for the local player, if spawned.
    pub fn player_entity(&self) -> Option<Entity> {
        self.player_entity
    }

    /// Retrieves the ECS `Entity` corresponding to a Minecraft entity ID.
    pub fn get_mob_entity(&self, minecraft_id: u32) -> Option<Entity> {
        self.mob_map.get(&minecraft_id).copied()
    }
}
