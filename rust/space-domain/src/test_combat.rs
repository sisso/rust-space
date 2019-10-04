use std::collections::HashMap;

use crate::game::ship::ship_internals::*;
use crate::game::ship::ship_combat::*;

pub fn run() {
    let mut components = Components::new();

    let engine_id = ComponentId(0);
    let fuel_tank_id = ComponentId(1);
    let bridge_id = ComponentId(2);
    let quarters_id = ComponentId(3);
    let engine_room_id = ComponentId(4);
    let reactor_id = ComponentId(5);
    let gaus_weapon_id = ComponentId(6);
    let lazer_weapon_id = ComponentId(7);
    let plasma_weapon_id = ComponentId(8);

    let mut engine = Component::new(engine_id, ComponentType::Engine);
    engine.crew_require = 10.0;
    engine.thrust = 200.0;
    engine.weight = 1000;
    engine.fuel_consume = 0.062;
    engine.width = 10;
    engine.engineer_require = 10.0;
    components.add(engine);

    let mut fuel_tank = Component::new(fuel_tank_id, ComponentType::FuelTank);
    fuel_tank.crew_require = 0.5;
    fuel_tank.weight = 100;
    fuel_tank.fuel_hold = 5000.0;
    fuel_tank.width = 1;
    fuel_tank.engineer_require = 0.1;
    components.add(fuel_tank);

    let mut bridge = Component::new(bridge_id, ComponentType::Bridge);
    bridge.crew_require = 5.0;
    bridge.engineer_require = 1.0;
    bridge.weight = 50;
    bridge.width = 1;
    components.add(bridge);

    let mut quarters = Component::new(quarters_id, ComponentType::Quarter);
    quarters.crew_provide = 50.0;
    quarters.engineer_require = 0.1;
    quarters.weight = 50;
    quarters.width = 1;
    components.add(quarters);

    let mut enginer_room = Component::new(engine_room_id, ComponentType::Engineer);
    enginer_room.crew_require = 5.0;
    enginer_room.engineer_provide = 10.0;
    enginer_room.weight = 50;
    enginer_room.width = 1;
    components.add(enginer_room);

    let mut reactor = Component::new(reactor_id, ComponentType::PowerGenerator);
    reactor.crew_require = 5.0;
    reactor.engineer_require = 5.0;
    reactor.weight = 50;
    reactor.power_generate = 5.0;
    reactor.width = 1;
    components.add(reactor);

    let mut gaus_weapon = Component::new(gaus_weapon_id, ComponentType::Weapon);
    gaus_weapon.crew_require = 5.0;
    gaus_weapon.engineer_require = 1.0;
    gaus_weapon.weight = 50;
    gaus_weapon.width = 1;
    gaus_weapon.power_require = 1.0;
    gaus_weapon.weapon = Some(
        Weapon {
            damage: Damage(1),
            reload: 1.0,
            rounds: 3,
            damage_type: WeaponDamageType::Explosive,
        }
    );
    components.add(gaus_weapon);

    let mut lazer_weapon = Component::new(lazer_weapon_id, ComponentType::Weapon);
    lazer_weapon.crew_require = 5.0;
    lazer_weapon.engineer_require = 1.0;
    lazer_weapon.weight = 50;
    lazer_weapon.width = 1;
    lazer_weapon.power_require = 1.0;
    lazer_weapon.weapon = Some(
        Weapon {
            damage: Damage(4),
            reload: 2.0,
            rounds: 1,
            damage_type: WeaponDamageType::Penetration,
        }
    );
    components.add(lazer_weapon);

    let mut plasma_weapon = Component::new(plasma_weapon_id, ComponentType::Weapon);
    plasma_weapon.crew_require = 5.0;
    plasma_weapon.engineer_require = 1.0;
    plasma_weapon.weight = 50;
    plasma_weapon.width = 1;
    plasma_weapon.power_require = 3.0;
    plasma_weapon.weapon = Some(
        Weapon {
            damage: Damage(4),
            reload: 2.0,
            rounds: 1,
            damage_type: WeaponDamageType::Explosive,
        }
    );
    components.add(plasma_weapon);

    let mut base_components = HashMap::new();
    base_components.insert(bridge_id, 1);
    base_components.insert(engine_id, 1);
    base_components.insert(fuel_tank_id, 1);
    base_components.insert(reactor_id, 1);
    base_components.insert(engine_room_id, 2);
    base_components.insert(quarters_id, 1);

    let mut ship_components1 = base_components.clone();
    ship_components1.insert(gaus_weapon_id, 3);

    let mut ship_components2 = base_components.clone();
    ship_components2.insert(plasma_weapon_id, 2);
    ship_components2.insert(reactor_id, 2);
    ship_components2.insert(engine_room_id, 3);
    ship_components2.insert(quarters_id, 2);

    let specs1 = ShipSpec::new(&components, ship_components1, 3);
    let valid1 = specs1.is_valid();
    println!("valid: {:?}", valid1);

    let specs2 = ShipSpec::new(&components, ship_components2, 3);
    let valid2 = specs2.is_valid();
    println!("valid: {:?}", valid2);


    if valid1.is_err() || valid2.is_err() {
        panic!();
    }

    let team_red = TeamId(0);
    let team_blue = TeamId(1);

    let ship_1_id = ShipInstanceId(0);
    let mut ship1 = ShipInstance::new(&components, ship_1_id, specs1, team_red);

    let ship_2_id = ShipInstanceId(1);
    let mut ship2 = ShipInstance::new(&components, ship_2_id, specs2, team_blue);

    println!("ship: {:?}", ship1);
    println!("ship: {:?}", ship2);

    let mut combat_ctx = CombatContext::new(&components);
    combat_ctx.add_ship(&mut ship1);
    combat_ctx.add_ship(&mut ship2);
    combat_ctx.set_time(0.5, 0.5);
    combat_ctx.set_distance(ship_1_id, ship_2_id, 1.0);

    let mut time = 0.0;
    let delta = 1.0;

    for round in 0..200 {
        time += delta;

        combat_ctx.set_time(delta, time);
        combat_ctx.set_distance(ship_1_id, ship_2_id, 1.0);

        println!("-----------------------------------------------------------");
        println!("round {}", round);
        println!("-----------------------------------------------------------");

        let mut logs = vec![];
        let damages_to_apply = Combat::execute(&mut combat_ctx, &mut logs);
        let ships = combat_ctx.get_ships();

        println!("ship: {:?}", ships.get(0));
        println!("ship: {:?}", ships.get(1));

        let finish = logs.iter().find(|i| {
           match i {
               CombatLog::ShipDestroyed { .. } => true,
               _ => false
           }
        }).is_some();

        for log in logs {
            println!("- {:?}", log);
        }

        println!("{}", print_hull(ships.get(0).unwrap()));
        println!("{}", print_hull(ships.get(1).unwrap()));

        if finish {
            break;
        }

//        println!("<press to continue>");
//        print!("{}[2J", 27 as char);
//        let _ = std::io::stdin().read_line(&mut String::new());
    }
}

fn print_hull(ship: &ShipInstance) -> String {
    let mut buffer = String::new();
    let mut index = 0;

    for layer in 0..ship.spec.armor.height {
        for i in 0..ship.spec.armor.width {
            if ship.armor_damage.contains(&ArmorIndex(index)) {
                buffer.push('.');
            } else {
                buffer.push('#');
            }

            index += 1;
        }
        buffer.push('\n');
    }

    buffer
}
