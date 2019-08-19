use std::collections::HashMap;

use crate::game::ship_internals::*;
use crate::game::ship_combat::*;

pub fn run() {
    let mut components = Components::new();

    let engine_id = components.next_id();
    let fuel_tank_id = components.next_id();
    let bridge_id = components.next_id();
    let quarters_id = components.next_id();
    let engine_room_id = components.next_id();
    let reactor_id = components.next_id();
    let gaus_weapon_id = components.next_id();

    let mut engine = Component::new(engine_id, ComponentType::Engine);
    engine.crew_require = 10.0;
    engine.thrust = 200.0;
    engine.weight = 1000.0;
    engine.fuel_consume = 0.062;
    engine.width = 10.0;
    engine.engineer_require = 10.0;
    components.add(engine);

    let mut fuel_tank = Component::new(fuel_tank_id, ComponentType::FuelTank);
    fuel_tank.crew_require = 0.5;
    fuel_tank.weight = 100.0;
    fuel_tank.fuel_hold = 5000.0;
    fuel_tank.width = 1.0;
    fuel_tank.engineer_require = 0.1;
    components.add(fuel_tank);

    let mut bridge = Component::new(bridge_id, ComponentType::Bridge);
    bridge.crew_require = 5.0;
    bridge.engineer_require = 1.0;
    bridge.weight = 50.0;
    bridge.width = 1.0;
    components.add(bridge);

    let mut quarters = Component::new(quarters_id, ComponentType::Quarter);
    quarters.crew_provide = 50.0;
    quarters.engineer_require = 0.1;
    quarters.weight = 50.0;
    quarters.width = 1.0;
    components.add(quarters);

    let mut enginer_room = Component::new(engine_room_id, ComponentType::Engineer);
    enginer_room.crew_require = 5.0;
    enginer_room.engineer_provide = 10.0;
    enginer_room.weight = 50.0;
    enginer_room.width = 1.0;
    components.add(enginer_room);

    let mut reactor = Component::new(reactor_id, ComponentType::PowerGenerator);
    reactor.crew_require = 5.0;
    reactor.engineer_require = 5.0;
    reactor.weight = 50.0;
    reactor.power_generate = 5.0;
    reactor.width = 1.0;
    components.add(reactor);

    let mut gaus_weapon = Component::new(gaus_weapon_id, ComponentType::Weapon);
    gaus_weapon.crew_require = 5.0;
    gaus_weapon.engineer_require = 1.0;
    gaus_weapon.weight = 50.0;
    gaus_weapon.width = 1.0;
    gaus_weapon.power_require = 1.0;
    gaus_weapon.weapon = Some(
        Weapon {
            damage: 1.0,
            reload: 1.0,
            rounds: 1,
            damage_type: WeaponDamageType::Explosive,
        }
    );
    components.add(gaus_weapon);

    let mut ship_components = HashMap::new();
    ship_components.insert(bridge_id, 1);
    ship_components.insert(engine_id, 1);
    ship_components.insert(fuel_tank_id, 1);
    ship_components.insert(gaus_weapon_id, 3);
    ship_components.insert(reactor_id, 1);
    ship_components.insert(engine_room_id, 2);
    ship_components.insert(quarters_id, 1);

    let width = compute_width(&components, &ship_components);

    let armor = Armor {
        width: width,
        height: 3,
    };

    let stats = ShipSpec::compute_ship_stats(&components, &ship_components, &armor);
    let valid = ShipSpec::is_valid(&stats);

    println!("stats: {:?}", stats);
    println!("valid: {:?}", valid);

    if valid.is_err() {
        panic!();
    }

    let ship_1_id = ShipInstanceId(0);
    let ship1 = ShipInstance {
        id: ship_1_id,
        spec: ShipSpec {
            armor: armor.clone(),
            components: ship_components.clone(),
            stats: stats.clone(),
        },
        armor_damage: Default::default(),
        component_damage: Default::default()
    };

    let ship_2_id = ShipInstanceId(0);
    let ship2 = ShipInstance {
        id: ship_2_id,
        spec: ShipSpec {
            armor: armor,
            components: ship_components,
            stats,
        },
        armor_damage: Default::default(),
        component_damage: Default::default()
    };

    let mut combat = ShipCombat::new(ship1, ship2);
    combat.set_distance(ship_1_id, ship_2_id, 1.0);
    combat.tick(0.1, 0.1);
}
