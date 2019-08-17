use std::collections::{HashMap, HashSet};
use crate::utils::{Log, NextId};

#[derive(Clone,Debug)]
struct Armor {
    width: u32,
    height: u32,
    damage: HashSet<u32>
}

impl Armor {
    pub fn weight(&self) -> f32 {
        self.width as f32 * self.height as f32  * 0.5
    }
}

#[derive(Copy,Clone,Debug)]
pub enum ComponentType {
    Engine,
    PowerGenerator,
    Quarter,
    Engineer,
    Bridge,
    FuelTank,
    Weapon,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentId(pub u32);

#[derive(Clone,Debug)]
pub enum Weapon {
    Gaus {
        damage: f32,
        reload: f32,
    }
}

#[derive(Clone,Debug)]
pub struct Component {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub weapon: Option<Weapon>,
    pub thrust: f32,
    pub weight: f32,
    pub crew_require: f32,
    pub crew_provide: f32,
    pub power_require: f32,
    pub power_generate: f32,
    pub engineer_provide: f32,
    pub engineer_require: f32,
    pub fuel_consume: f32,
    pub width: f32,
    pub fuel_hold: f32,
}

impl Component {
    pub fn new(id: ComponentId, component_type: ComponentType) -> Self {
        Component {
            id: id,
            component_type: component_type,
            weapon: None,
            thrust: 0.0,
            weight: 0.0,
            crew_require: 0.0,
            crew_provide: 0.0,
            power_require: 0.0,
            power_generate: 0.0,
            engineer_provide: 0.0,
            engineer_require: 0.0,
            fuel_consume: 0.0,
            width: 0.0,
            fuel_hold: 0.0,
        }
    }
}

#[derive(Clone,Debug)]
struct ComponentItem {
    damage: u32,
    max_damage: u32,
    working: u32,
}

#[derive(Clone,Debug)]
struct ShipComponents {
    /// List of components, value is amount
    components: HashMap<ComponentId, u32>,
    total_weight: f32,
}

#[derive(Clone,Debug)]
struct ShipInternal {
    armor: Armor,
    components: ShipComponents
}

pub struct Components {
    next_id: NextId,
    index: HashMap<ComponentId, Component>
}

impl Components {
    pub fn new() -> Self {
        Components {
            next_id: NextId::new(),
            index: Default::default()
        }
    }

    pub fn next_id(&mut self) -> ComponentId {
        ComponentId(self.next_id.next())
    }

    pub fn add(&mut self, component: Component) {
        if self.index.contains_key(&component.id) {
            panic!();
        }

        Log::info("components", &format!("{:?} added {:?}", component.id, component));
        self.index.insert(component.id, component);
    }

    pub fn get(&self, component_id: &ComponentId) -> &Component {
        self.index.get(component_id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_ship_test() {
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
           Weapon::Gaus {
               damage: 1.0,
               reload: 1.0,
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
        let weight = compute_weight(&components, &ship_components, width, 2);
        let valid = is_valid(&components, &ship_components);

        println!("width: {:?}", width);
        println!("weight: {:?}", weight);
        println!("valid: {:?}", valid);

        let ship1 = ShipInternal {
            armor: Armor {
                width: 10,
                height: 3,
                damage: Default::default()
            },
            components: ShipComponents {
                components: ship_components,
                total_weight: 0.0
            }
        };

        assert!(false);
    }
}

fn compute_width(components: &Components, ship_components: &HashMap<ComponentId, u32>) -> u32 {
    let sum = ship_components.iter().map(|(component_id, amount)| {
        components.get(component_id).width * *amount as f32
    }).sum();

    round_width(sum)
}

#[derive(Debug, Clone)]
pub enum ShipComponentsValidation {
    NoBridge,
    NeedCrew { amount: f32 },
    NeedPower { amount: f32 },
    NeedFuel { amount: f32 },
    NeedEngineer { amount: f32 },
}

fn is_valid(components: &Components, ship_components: &HashMap<ComponentId, u32>) -> Result<(), Vec<ShipComponentsValidation>>{
    let mut errors = vec![];

    let mut has_bridge = false;

    let mut power = 0.0;
    let mut crew = 0.0;
    let mut engineer = 0.0;

    let mut mapped: HashMap<ComponentId, (&u32, &Component)> = HashMap::new();
    for (id, amount) in ship_components {
        let famount = *amount as f32;
        let component = components.get(id);

        if let ComponentType::Bridge = component.component_type {
            has_bridge = true;
        }

        power += component.power_generate * famount - component.power_require * famount;
        crew += component.crew_provide * famount - component.crew_require * famount;
        engineer += component.engineer_provide * famount - component.engineer_require * famount;

        mapped.insert(*id, (amount, component));
    }

    if !has_bridge {
        errors.push(ShipComponentsValidation::NoBridge);
    }

    if power < 0.0 {
        errors.push(ShipComponentsValidation::NeedPower { amount: -power });
    }

    if crew < 0.0 {
        errors.push(ShipComponentsValidation::NeedCrew { amount: -crew });
    }

    if engineer < 0.0 {
        errors.push(ShipComponentsValidation::NeedEngineer { amount: -engineer });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn compute_weight(components: &Components, ship_components: &HashMap<ComponentId, u32>, armor_width: u32, armor_height: u32) -> u32 {
    let sum: f32 = ship_components.iter().map(|(component_id, amount)| {
        components.get(component_id).weight * *amount as f32
    }).sum();

    let armor_sum = (armor_width * armor_height * 10) as f32;

    round_weight(sum + armor_sum)
}

fn round_width(value: f32) -> u32 {
    value.ceil() as u32
}

fn round_weight(value: f32) -> u32 {
    value.ceil() as u32
}
