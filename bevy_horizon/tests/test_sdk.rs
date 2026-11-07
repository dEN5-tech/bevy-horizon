use bevy_horizon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_components_and_math() {
    let p1 = Position::new(0.0, 10.0, 0.0);
    let p2 = Position::new(3.0, 14.0, 0.0);
    assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-5);
    assert!((p1.distance_sq(&p2) - 25.0).abs() < 1e-5);

    let v = Velocity::new(0.0, 3.0, 4.0);
    assert!((v.length() - 5.0).abs() < 1e-5);
    let norm = v.normalize();
    assert!((norm[1] - 0.6).abs() < 1e-5);
    assert!((norm[2] - 0.8).abs() < 1e-5);

    let mut health = Health::new(15.0, 20.0);
    assert!(health.is_alive());
    assert!((health.percentage() - 0.75).abs() < 1e-5);
    health.damage(5.0);
    assert_eq!(health.current, 10.0);
    health.heal(20.0);
    assert_eq!(health.current, 20.0);

    let rot = Rotation::new(0.0, 0.0);
    let forward = rot.forward_vector();
    assert!((forward[2] - 1.0).abs() < 1e-5);

    let pedestal = PedestalBlock::new("minecraft:diamond_block", [10, 64, -20]);
    let center = pedestal.anchor_center();
    assert_eq!(center, [10.5, 65.0, -19.5]);

    let mut orbit = OrbitingEntity::new(center, 4.0, 1.0);
    orbit.advance(1.57079632679); // ~PI/2
    let pos = orbit.calculate_position(0.0);
    assert!((pos[0] - center[0]).abs() < 1e-2);
    assert!((pos[2] - (center[2] + 4.0)).abs() < 1e-2);
}

#[test]
fn test_bundles_and_archetype_queries() {
    let mut world = World::new();

    let player = world
        .spawn(LocalPlayerBundle::new(1, [10.0, 70.0, 10.0], [0.0, 0.0], 20.0, 20.0))
        .id();

    let zombie = world
        .spawn((
            MinecraftMobBundle::new(2, "minecraft:zombie", [12.0, 70.0, 10.0], 20.0, 20.0),
            Zombie,
            Undead,
            HostileMob,
        ))
        .id();

    let skeleton = world
        .spawn((
            MinecraftMobBundle::new(3, "minecraft:skeleton", [14.0, 70.0, 10.0], 20.0, 20.0),
            Skeleton,
            Undead,
            HostileMob,
        ))
        .id();

    let cow = world
        .spawn((
            MinecraftMobBundle::new(4, "minecraft:cow", [16.0, 70.0, 10.0], 10.0, 10.0),
            PassiveAnimal,
        ))
        .id();

    assert!(world.get_entity(player).unwrap().get::<LocalPlayer>().is_some());
    assert!(world.get_entity(zombie).unwrap().get::<Zombie>().is_some());
    assert!(world.get_entity(skeleton).unwrap().get::<Skeleton>().is_some());
    assert!(world.get_entity(cow).unwrap().get::<PassiveAnimal>().is_some());

    // Query Undead
    let mut undead_query = world.query_filtered::<&MinecraftEntityId, With<Undead>>();
    let undead_ids: Vec<u32> = undead_query.iter(&world).map(|id| id.get()).collect();
    assert_eq!(undead_ids.len(), 2);
    assert!(undead_ids.contains(&2));
    assert!(undead_ids.contains(&3));
}

#[test]
fn test_resources_and_commands() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    HorizonPlugin::build(&mut world, &mut schedule);

    // Verify Clock
    {
        let mut clock = world.resource_mut::<HorizonClock>();
        clock.advance(0.05);
        assert_eq!(clock.tick, 1);
        assert!((clock.time_secs - 0.05).abs() < 1e-6);
    }

    // Verify Outbox commands
    {
        let mut outbox = world.resource_mut::<HorizonOutboxCommands>();
        outbox.teleport_mob(0, 42, [100.0, 64.0, -200.0]);
        outbox.place_block(0, [100, 63, -200], "minecraft:beacon");
        outbox.push_mob(0, 42, [0.0, 1.5, 0.0]);
        outbox.action_bar("Target locked!");
        outbox.chat("Hello from Bevy ECS!");
        assert_eq!(outbox.len(), 5);
        let drained = outbox.drain_commands();
        assert_eq!(drained.len(), 5);
        assert!(outbox.is_empty());
    }

    // Verify HUD buffer
    {
        let mut hud = world.resource_mut::<HudDrawBuffer>();
        hud.draw_text("FPS: 144", 10.0, 10.0, 0xFFFFFF, true);
        hud.fill_rect(5.0, 5.0, 100.0, 30.0, 0x80000000);
        assert_eq!(hud.len(), 2);
        let cmds = hud.drain_commands();
        assert_eq!(cmds.len(), 2);
        assert!(hud.is_empty());
    }
}

#[test]
fn test_events_and_observers() {
    let mut world = World::new();

    let interact_flag = Arc::new(AtomicBool::new(false));
    let interact_flag_clone = interact_flag.clone();
    world.add_observer(move |event: On<EntityInteractEvent>| {
        if event.entity_id == 77 {
            interact_flag_clone.store(true, Ordering::SeqCst);
        }
    });

    let key_pressed_counter = Arc::new(AtomicU32::new(0));
    let key_pressed_counter_clone = key_pressed_counter.clone();
    world.add_observer(move |event: On<HorizonKeyEvent>| {
        if event.pressed && event.key == "key.keyboard.r" {
            key_pressed_counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    world.trigger(EntityInteractEvent::new(77, "minecraft:villager", None));
    assert!(interact_flag.load(Ordering::SeqCst));

    world.trigger(HorizonKeyEvent::new("key.keyboard.r", true));
    world.trigger(HorizonKeyEvent::new("key.keyboard.r", false));
    world.trigger(HorizonKeyEvent::new("key.keyboard.space", true));
    assert_eq!(key_pressed_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_horizon_bridge_lifecycle() {
    let mut bridge = HorizonBridge::new();

    // 1. Ingest player
    let player = bridge.ingest_player(100, [0.0, 64.0, 0.0], [90.0, 0.0], 20.0, 20.0);
    assert_eq!(bridge.player_entity(), Some(player));

    // 2. Ingest mobs
    bridge.ingest_mob(201, "minecraft:zombie", [5.0, 64.0, 0.0], 20.0, 20.0);
    bridge.ingest_mob(202, "minecraft:cow", [-5.0, 64.0, 0.0], 10.0, 10.0);

    let zombie_entity = bridge.get_mob_entity(201).unwrap();
    let cow_entity = bridge.get_mob_entity(202).unwrap();

    assert!(bridge.world.get_entity(zombie_entity).unwrap().get::<Zombie>().is_some());
    assert!(bridge.world.get_entity(cow_entity).unwrap().get::<PassiveAnimal>().is_some());

    // 3. Despawn synchronization
    bridge.sync_despawned_mobs(&[201]);
    assert!(bridge.get_mob_entity(201).is_some());
    assert!(bridge.get_mob_entity(202).is_none());

    // 4. Spawn Orbiting Entity
    let orbit_bundle = PedestalOrbitBundle::new("minecraft:gold_block", [0, 64, 0], 3.0, 2.0);
    let orbit_ent = bridge.world.spawn(orbit_bundle).id();

    // 5. Step tick
    bridge.step_tick(0.05);

    let orbit_pos = *bridge.world.get_entity(orbit_ent).unwrap().get::<Position>().unwrap();
    assert!(orbit_pos.x != 0.0 || orbit_pos.z != 0.0);

    // 6. Test outbox emission during tick
    bridge.world.resource_mut::<HorizonOutboxCommands>().action_bar("Tick OK");
    let outbox = bridge.drain_outbox();
    assert_eq!(outbox.len(), 1);

    // 7. Test HUD stepping
    bridge.step_hud(0.016);
    let _ = bridge.drain_hud();
}

#[test]
fn test_package_hnmod() {
    let tmp = std::env::temp_dir().join("bevy_horizon_test_pkg");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let compat_path = tmp.join("compatibility.json");
    let compat_content = r#"{
      "game": "26.3",
      "hooks": 359,
      "horizon_revision": "66f3981f522b24c250b21523a70080965b940424",
      "horizon_mixins_revision": "e8cd0aa38cffef49cb2e088ef716baa107148c02"
    }"#;
    fs::write(&compat_path, compat_content).unwrap();

    let mixins_dir = tmp.join("mixins").join("test_mod");
    fs::create_dir_all(&mixins_dir).unwrap();
    fs::write(mixins_dir.join("runtime.rs"), "pub fn test() {}").unwrap();
    fs::write(mixins_dir.join("horizon-mixin.toml"), "[mod]\nid = \"test_mod\"\n").unwrap();

    let out_hnmod = tmp.join("test_mod.hnmod");
    let res = package_hnmod(&compat_path, &mixins_dir, &out_hnmod).expect("Packaging failed");

    assert!(out_hnmod.is_file());
    assert!(res.size_bytes > 0);
    assert_eq!(res.file_count, 3); // horizon-native.json, runtime.rs, horizon-mixin.toml

    let _ = fs::remove_dir_all(&tmp);
}
