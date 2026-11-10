# ⚡ bevy-horizon

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/bevy_horizon.svg?style=for-the-badge&logo=rust)](https://crates.io/crates/bevy_horizon)
[![Docs.rs](https://docs.rs/bevy_horizon/badge.svg?style=for-the-badge&logo=docsdotrs)](https://docs.rs/bevy_horizon)
[![Bevy Version](https://img.shields.io/badge/Bevy-0.20-568bf5.svg?style=for-the-badge&logo=bevy)](https://bevyengine.org)
[![Minecraft Horizon](https://img.shields.io/badge/Minecraft-26.3%20Native-00aa00.svg?style=for-the-badge&logo=minecraft)](https://github.com/horizon-rust/horizon)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-orange.svg?style=for-the-badge)](#license)
[![Build Status](https://img.shields.io/badge/tests-passing-success.svg?style=for-the-badge&logo=githubactions)](#testing)

**The first-class Bevy ECS runtime adapter, AST type generator, and build toolkit for Minecraft Horizon native mods.**

[Architecture](#-architecture) • [Features](#-key-features) • [Quickstart](#-quickstart) • [Reference Mod](#-example-pedestal-orbit-mod) • [Build Tooling](#-build-tooling--hnmod-packager) • [AST Typegen](#-ast-typegen-pipeline)

</div>

---

## 🌟 Overview

**`bevy-horizon`** bridges **Minecraft Horizon** (the high-performance, native Rust client for Minecraft 26.3) with **Bevy Engine**'s data-oriented ECS architecture (`bevy_ecs 0.20`).

Instead of writing imperative, procedural mixin hooks with raw mutable pointers and primitive arrays (`&mut Game<'_>`, `[f64; 3]`), `bevy-horizon` empowers you to write clean, declarative, multi-threaded Bevy systems with `Query`, `Res`, `Commands`, and typed Minecraft archetypes:

```rust
use bevy_horizon::prelude::*;

/// Pure Bevy System: Animate all zombies orbiting smoothly around the player
fn orbit_zombies_system(
    player_q: Query<&Position, With<LocalPlayer>>,
    mut zombies_q: Query<(&MinecraftEntityId, &mut Position), With<Zombie>>,
    mut outbox: ResMut<HorizonOutboxCommands>,
    clock: Res<HorizonClock>,
) {
    let Ok(player_pos) = player_q.single() else { return };
    let angle = clock.tick as f64 * 0.05;

    for (id, mut pos) in zombies_q.iter_mut() {
        pos.x = player_pos.x + 4.0 * angle.cos();
        pos.z = player_pos.z + 4.0 * angle.sin();
        
        // Dispatched atomically to the Minecraft client at the end of the tick
        outbox.teleport_mob(id.0, pos.to_array());
    }
}
```

---

## 🏗️ Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Minecraft Horizon Client                        │
│             &mut Game, Raw Mobs, Player, HUD Batch, Packets            │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 1. State Ingestion & Despawn Tracking
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   bevy_horizon::HorizonBridge                          │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │                         Bevy World                             │   │
│   │                                                                │   │
│   │  [LocalPlayer]           [MinecraftMob]       [PedestalBlock]  │   │
│   │  - Position              - MinecraftEntityId  - Position       │   │
│   │  - Rotation              - EntityKind         - BlockType      │   │
│   │  - Health                - Zombie / Undead    - OrbitingEntity │   │
│   │                                                                │   │
│   │  [Resources]             [Event Triggers]     [Outbox Commands]│   │
│   │  - HorizonClock          - HorizonKeyEvent    - TeleportMob    │   │
│   │  - ScreenState           - EntityInteractEvent- PlaceBlock     │   │
│   │  - HudDrawBuffer         - PlayerTickEvent    - PushVelocity   │   │
│   └────────────────────────────────────────────────────────────────┘   │
│                                   │                                    │
│                                   │ 2. Bevy Schedule Run (Parallel)    │
│                                   ▼                                    │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │                        Bevy Schedule                           │   │
│   │    orbit_kinematics_system | user_mod_systems | hud_render     │   │
│   └────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 3. Egress Flush (Atomic Dispatch)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        Horizon Native Hooks                            │
│           game.native_teleport_mob, native_push, native_fill           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## ✨ Key Features

- 🎮 **Pure Bevy ECS Paradigm**: Leverage `Query`, `Res`, `ResMut`, `Entity`, `Bundle`, and `Observer` systems inside Minecraft.
- 🔄 **Bidirectional Bridge Engine**: Synchronizes living mobs, player transforms, item stacks, and HUD batches without engine overhead.
- 🏷️ **Archetype Classification**: Auto-tags raw mobs (`"minecraft:zombie"`, `"minecraft:skeleton"`, `"minecraft:cow"`) with typed markers (`Zombie`, `Skeleton`, `Creeper`, `PassiveAnimal`, `HostileMob`, `Undead`).
- ⚡ **Offline Headless Testing**: 100% testable logic using `cargo test` in **0.00 seconds** without launching Minecraft or mock servers.
- 📦 **Native Packager (`build.rs -> .hnmod`)**: Built-in deterministic `.hnmod` packager that produces compliant mod archives ready for the Horizon Launcher.
- 🧬 **AST Typegen (`horizon_bevy_gen`)**: Rust AST inspector & generator (inspired by [`rusty-ast`](https://github.com/katsuhirohonda/rusty-ast)) to reflect any Horizon engine update into typed Bevy components.

---

## 🚀 Quickstart

### 1. Add `bevy_horizon` to your `Cargo.toml`

```toml
[dependencies]
bevy_horizon = "0.1.0"
bevy_ecs = { version = "0.20", default-features = false }
```

### 2. Create your Bevy System

```rust
use bevy_horizon::prelude::*;

fn bounce_passive_animals(
    mut animals: Query<&mut Position, (With<PassiveAnimal>, Without<LocalPlayer>)>,
    time: Res<HorizonClock>,
) {
    for mut pos in animals.iter_mut() {
        pos.y += (time.tick as f64 * 0.1).sin() * 0.05;
    }
}
```

### 3. Initialize the Bridge in Horizon Mixin

```rust
use bevy_horizon::prelude::*;

fn my_mod_player_tick(game: &mut crate::game::Game<'_>) {
    let mut bridge = HorizonBridge::new();
    
    // 1. Ingest Player
    if let Some(player) = game.native_player() {
        bridge.ingest_player([player.x, player.y, player.z], player.yaw, player.pitch, 20.0, 20.0);
    }
    
    // 2. Ingest Nearby Mobs
    let mobs = game.native_mobs(0);
    bridge.ingest_mobs(mobs.into_iter().map(|m| (m.id, m.kind, m.pos, m.health, m.max_health)));
    
    // 3. Run Systems
    bridge.schedule.add_systems(bounce_passive_animals);
    bridge.run_schedule();
    
    // 4. Drain & Apply Outbox Commands
    let outbox = bridge.drain_commands();
    for (id, pos) in outbox.teleports {
        game.native_teleport_mob(0, id, pos);
    }
}
```

---


## 🔮 Example: Pedestal Orbit Mod

The included reference mod [`pedestal_orbit`](./pedestal_orbit) demonstrates how to build an interactive RPG totem mechanic:

1. **Block Selection**: Player cycles target block types (`Diamond`, `Gold`, `Obsidian`, `Beacon`, `Emerald`) via hotkey.
2. **Anchor Placement**: Places the pedestal block in the world.
3. **Orbiting Entity**: Binds a captured mob or familiar that smoothly orbits on top of the block with sinusoidal levitation bobbing ($y = y_0 + A \sin(\omega t)$).

Run the tests:
```bash
cargo test -p pedestal_orbit
```

---

## 🛠️ Build Tooling & `.hnmod` Packager

Package your Horizon native mod directly from `build.rs` or custom script:

```rust
// build.rs
use bevy_horizon::build_tool::package_hnmod;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=mixins/");
    
    package_hnmod(
        Path::new("manifest.json"),
        Path::new("mixins/my_mod"),
        Path::new("target/my_mod-0.1.0.hnmod"),
    ).expect("Failed to package .hnmod");
}
```

---

## 🧬 AST Typegen Pipeline

`bevy-horizon` includes a standalone AST code generator in [`tools/horizon_bevy_gen`](./tools/horizon_bevy_gen):

### Inspect Horizon Rust AST (JSON output)
```bash
cargo run -p horizon_bevy_gen -- inspect path/to/horizon/native_hooks.rs
```

### Generate fresh Bevy ECS Crate from Horizon Client source
```bash
cargo run -p horizon_bevy_gen -- generate path/to/horizon/crates/horizon-client ./generated_bevy_horizon
```

---

## 🧪 Testing

Run all unit and integration tests across the entire workspace:

```bash
cargo test --workspace
```

---

## 🏷️ Popular Topics & SEO Tags

`horizon` • `minecraft` • `bevy` • `bevy-ecs` • `rust` • `minecraft-mod` • `horizon-client` • `native-mods` • `game-development` • `gamedev` • `game-engine` • `modding` • `data-oriented` • `code-generation` • `ast` • `ecs`

---

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
