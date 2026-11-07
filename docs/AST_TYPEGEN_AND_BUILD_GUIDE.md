# Bevy-Horizon SDK: AST Typegen, Archetype Engine & Build Guide

## 1. Executive Architectural Overview

The **Bevy-Horizon SDK** provides an ergonomic, production-grade bridge between **Horizon** (the high-performance Rust Minecraft 26.3 client) and **Bevy ECS** (`bevy_ecs 0.20`).

Horizon executes Minecraft simulation and rendering natively in Rust. To extend the client without recompiling the entire engine or introducing unstable ABI boundaries, Horizon uses compile-time mixin injection managed by `horizon-mixins` and `rusty-ast`. `bevy_horizon` introduces the headless data-oriented ECS paradigm to this ecosystem:

```text
+-----------------------------------------------------------------------------------+
|                            Horizon Minecraft Client                               |
|                                                                                   |
|  +--------------------+    +--------------------+    +-------------------------+  |
|  | Game::native_player|    |Game::native_interact|   |Game::native_hud         |  |
|  | Tick (20 Hz)       |    |Entity (Input)      |    |Render Pass (VSync)      |  |
|  +---------+----------+    +---------+----------+    +------------+------------+  |
+------------|-------------------------|----------------------------|---------------+
             | (State Ingestion)       | (Input Events)             | (Buffer Drain)
             v                         v                            v
+-----------------------------------------------------------------------------------+
|                           bevy_horizon Bridge Runtime                             |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  |                               HorizonBridge                                 |  |
|  |                                                                             |  |
|  |  +-----------------------------------------------------------------------+  |  |
|  |  |                             Bevy World                                |  |  |
|  |  |                                                                       |  |  |
|  |  |  [LocalPlayer]           [OrbitingEntity]          [MinecraftMob]     |  |  |
|  |  |  - Position              - PedestalBlock           - MinecraftEntityId|  |  |
|  |  |  - Rotation              - OrbitingEntity          - EntityKind       |  |  |
|  |  |  - Velocity              - Position                - Zombie/Undead    |  |  |
|  |  |  - Health                                          - Health           |  |  |
|  |  |                                                                       |  |  |
|  |  |  [Resources]             [Event Triggers]          [Outbox Commands]  |  |  |
|  |  |  - HorizonClock          - HorizonKeyEvent         - TeleportMob      |  |  |
|  |  |  - ScreenState           - EntityInteractEvent     - PlaceBlock       |  |  |
|  |  |  - HudDrawBuffer         - PlayerTickEvent         - PushMob          |  |  |
|  |  +-----------------------------------------------------------------------+  |  |
|  |                                                                             |  |
|  |  +-----------------------------------------------------------------------+  |  |
|  |  |                           Bevy Schedule                               |  |  |
|  |  |  - orbit_kinematics_system                                            |  |  |
|  |  |  - velocity_integration_system                                        |  |  |
|  |  |  - user_mod_systems (Gaze, Impulse, HUD Telemetry, Stasis)            |  |  |
|  |  +-----------------------------------------------------------------------+  |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

---

## 2. Horizon Native AST Parsing & Injection Engine

### 2.1 The AST Parsing Pipeline (`rusty-ast` & `horizon-mixins`)

Horizon native mods are not dynamic libraries (`.dll`/`.so`). Instead, they are source-level mixin bundles injected directly into the Horizon client AST during compilation.

1. **Manifest Parsing**: The engine parses `mixins/<mod_id>/horizon-mixin.toml`.
2. **Crate Splicing**: Any referenced bundle crate (`[[crate]] id = "ecs"`) is attached to Horizon's Cargo workspace.
3. **Module Injection**:
   ```toml
   [[module]]
   id = "runtime"
   file = "crates/horizon-client/src/game.rs"
   name = "<mod_id>_runtime"
   visibility = "pub(crate)"
   source = "runtime.rs"
   ```
   The engine reads `runtime.rs`, parses it with `syn` into a `syn::ItemMod`, and splices `pub(crate) mod <mod_id>_runtime;` into `crates/horizon-client/src/game.rs`.
4. **Hook Point Injections**:
   ```toml
   [[injection]]
   id = "player_tick"
   kind = "HEAD"
   target = "horizon_client::game::native_hooks::Game<'_>::native_player_tick"
   helper = "player_tick.rs"
   bindings = [
     { name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" },
   ]
   ```
   `horizon-mixins` locates the target function in the client AST and generates a call to the helper function at the top (`HEAD`), bottom (`TAIL`), or around the invocation.

### 2.2 Thread-Local Kernel Isolation

Minecraft client threads are sensitive to lock contention. Running Bevy's ECS concurrently without lock blocking is achieved via `thread_local!` encapsulation:

```rust
use std::cell::RefCell;

thread_local! {
    static BRIDGE: RefCell<Option<HorizonBridge>> = const { RefCell::new(None) };
}

pub fn with_bridge<R>(f: impl FnOnce(&mut HorizonBridge) -> R) -> R {
    BRIDGE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let bridge = borrow.get_or_insert_with(HorizonBridge::new);
        f(bridge)
    })
}
```

---

## 3. Mapping Minecraft State to Bevy ECS Archetypes

### 3.1 Archetype Layout in `bevy_ecs 0.20`

Bevy ECS stores components in contiguous memory chunks called **Archetypes** and **Tables**. An entity's archetype is determined by the exact set of components it contains.

`bevy_horizon` defines the following core archetypes:

| Archetype | Components | Description |
|---|---|---|
| **Local Player** | `LocalPlayer`, `MinecraftEntityId`, `Position`, `Rotation`, `Velocity`, `Health` | Tracks the client's current position, view angles, and vitality. |
| **Living Mob** | `MinecraftEntityId`, `EntityKind`, `Position`, `Rotation`, `Velocity`, `Health` | Base archetype for any living Minecraft entity. |
| **Zombie / Undead** | Mob components + `Zombie`, `Undead`, `HostileMob` | Auto-classified marker components for query filtering. |
| **Skeleton / Undead** | Mob components + `Skeleton`, `Undead`, `HostileMob` | Auto-classified marker components. |
| **Creeper / Hostile** | Mob components + `Creeper`, `HostileMob` | Explosive hostile entity. |
| **Passive Animal** | Mob components + `PassiveAnimal` | Cows, sheep, pigs, chickens. |
| **Pedestal Orbit** | `PedestalBlock`, `OrbitingEntity`, `Position` | Anchored pedestal block with an entity orbiting around it. |

### 3.2 Ingestion & Despawn Synchronization

Every client tick, the native hook ingests active mobs and synchronizes removals:

```rust
// Ingest local player
bridge.ingest_player(player.id, [player.x, player.y, player.z], [player.yaw, player.pitch], player.health, player.max_health);

// Ingest surrounding mobs
let live_mobs = game.native_mobs(player.dimension);
let mut live_ids = Vec::with_capacity(live_mobs.len());

for mob in &live_mobs {
    live_ids.push(mob.id);
    bridge.ingest_mob(mob.id, &mob.kind, mob.pos, mob.health, mob.max_health);
}

// Automatically despawn entities that left the simulation distance
bridge.sync_despawned_mobs(&live_ids);
```

---

## 4. Writing a Bevy Plugin Mod with `HorizonPlugin`

### 4.1 Event Observers in Bevy 0.20

In `bevy_ecs 0.20`, events are triggered immediately using `world.trigger(...)` and observed via `world.add_observer(...)` using `On<T>`:

```rust
use bevy_horizon::prelude::*;

pub fn setup_pedestal_mod(bridge: &mut HorizonBridge) {
    // Register event observer for block placement
    bridge.world.add_observer(|event: On<BlockPlaceEvent>| {
        println!("Block placed at {:?}: {}", event.pos, event.block_type);
    });

    // Register event observer for player interaction (right-click)
    bridge.world.add_observer(|mut event: On<EntityInteractEvent>, mut outbox: ResMut<HorizonOutboxCommands>| {
        if !event.claimed {
            outbox.action_bar(format!("Interacted with mob: {}", event.kind));
            event.claim();
        }
    });

    // Register custom systems into the schedule
    bridge.schedule.add_systems(custom_pedestal_hud_system);
}
```

### 4.2 Kinematics: Sinusoidal Bobbing and Orbiting

The built-in `orbit_kinematics_system` computes circular trajectory with vertical harmonic bobbing:

$$\begin{aligned}
x(t) &= x_{\text{center}} + R \cdot \cos(\theta) \\
y(t) &= y_{\text{center}} + A \cdot \sin(\omega \cdot t) \\
z(t) &= z_{\text{center}} + R \cdot \sin(\theta)
\end{aligned}$$

Where:
- $R$ is the orbit radius (`radius`).
- $\theta$ is the advanced angular phase (`angle += speed * delta_secs`).
- $A$ is the bobbing amplitude (`bob_amplitude`, default `0.5`).
- $\omega$ is the bobbing angular frequency (`bob_frequency`, default `2.0`).

### 4.3 Draining Outbox Commands and HUD Primitives

Systems communicate side effects by writing to `HorizonOutboxCommands` or `HudDrawBuffer`. After running `schedule.run(&mut world)`:

```rust
// 1. Drain commands and dispatch to Minecraft hooks
for cmd in bridge.drain_outbox() {
    match cmd {
        OutboxCommand::TeleportMob { dimension, entity_id, pos } => {
            game.native_teleport_mob(dimension, entity_id, pos);
        }
        OutboxCommand::PlaceBlock { dimension, pos, block_id } => {
            game.native_place_block(dimension, pos, &block_id);
        }
        OutboxCommand::PushMob { dimension, entity_id, velocity } => {
            game.native_push_mob(dimension, entity_id, velocity);
        }
        OutboxCommand::ActionBar { message } => {
            let uuid = game.native_acting_id();
            let _ = game.native_action_bar(uuid, horizon_client::text::Text::literal(&message));
        }
        OutboxCommand::Chat { message } => {
            game.native_chat(&message);
        }
    }
}

// 2. Drain HUD buffer into native HudBatch
for cmd in bridge.drain_hud() {
    match cmd {
        HudDrawCommand::Text { text, x, y, color_rgb, shadow } => {
            game.native_text(batch, &text, [x, y], color_rgb, shadow);
        }
        HudDrawCommand::Rect { x, y, w, h, color_argb } => {
            game.native_fill(batch, [x, y, x + w, y + h], color_argb);
        }
    }
}
```

---

## 5. Build Pipeline, Typegen & Packaging (`.hnmod`)

### 5.1 The `.hnmod` Archive Specification

An `.hnmod` archive is a deterministic ZIP package containing:

1. `horizon-native.json`: Manifest describing engine compatibility.
2. `mixins/<mod_id>/horizon-mixin.toml`: Injection declaration.
3. `mixins/<mod_id>/**/*.rs`: Mixin helpers and module sources.
4. `mixins/<mod_id>/**/Cargo.toml`: ECS bundle dependency descriptors.
5. `mixins/<mod_id>/LICENSE`: Open-source license file.

### 5.2 Deterministic Build Tool (`package_hnmod`)

`bevy_horizon::package_hnmod` automates this packaging directly from `build.rs` or standard CLI scripts:

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=compatibility.json");
    println!("cargo:rerun-if-changed=mixins/");

    #[cfg(feature = "package")]
    {
        use bevy_horizon::package_hnmod;
        package_hnmod(
            "compatibility.json",
            "mixins/pedestal_orbit",
            "target/pedestal_orbit.hnmod",
        )
        .expect("Failed to package .hnmod");
    }
}
```

This guarantees complete bit-for-bit reproducible mod distribution ready for installation in `%APPDATA%/horizon/horizon-native-mods/`.
