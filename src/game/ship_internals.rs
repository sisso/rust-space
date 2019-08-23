use std::collections::{HashMap, HashSet};

use crate::utils::{Log, NextId};

#[derive(Clone,Debug)]
pub struct Armor {
    pub width: u32,
    pub height: u32,
}

impl Armor {
    pub fn new(width: u32, height: u32) -> Self {
        Armor { width, height }
    }

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ShipInstanceId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Damage(pub u32);


#[derive(Clone,Debug)]
pub enum WeaponDamageType {
    Explosive,
    Penetration
}

#[derive(Clone,Debug)]
pub struct Weapon {
    pub damage: Damage,
    pub reload: f32,
    pub rounds: u32,
    pub damage_type: WeaponDamageType
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
    pub damage: u32,
    pub max_damage: u32,
    pub working: u32,
}

#[derive(Clone,Debug)]
pub struct ShipSpec {
    pub armor: Armor,
    pub components: HashMap<ComponentId, u32>,
    pub stats: ShipStats,
}

impl ShipSpec {
    pub fn new(components: &Components, ship_components: HashMap<ComponentId, u32>, armor_height: u32) -> Self {
        let armor = Armor::new(compute_width(components, &ship_components), armor_height);
        let stats = ShipSpec::compute_ship_stats(components, &ship_components, &armor);
        ShipSpec { armor: armor, components: ship_components, stats: stats }
    }

    pub fn amount(&self, component_id: &ComponentId) -> Option<&u32> {
        self.components.get(component_id)
    }

    pub fn find_weapons(&self, components: &Components) -> Vec<ComponentId> {
        self.components
            .iter()
            .filter(|(id, _)| {
                let component = components.get(id);
                component.weapon.is_some()
            })
            .map(|(id, _)| {
                id.clone()
            })
            .collect()
    }

    pub fn compute_ship_stats(components: &Components, ship_components: &HashMap<ComponentId, u32>, armor: &Armor) -> ShipStats {
        let mut has_bridge = false;

        let mut power = 0.0;
        let mut crew = 0.0;
        let mut engineer = 0.0;
        let mut thrust: f32 = 0.0;

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
            thrust += component.thrust * famount;

            mapped.insert(*id, (amount, component));
        }

        let weight = compute_weight(components, ship_components, armor);

        ShipStats {
            bridge: has_bridge,
            total_weight: weight,
            total_width: compute_width(components, ship_components),
            power_balance: power,
            crew_balance: crew,
            engineer_balance: engineer,
            thrust: thrust,
            speed: 10.0 * thrust / (weight as f32),
        }
    }

    pub fn is_valid(&self) -> Result<(), Vec<ShipComponentsValidation>>{
        let stats = &self.stats;

        let mut errors = vec![];

        if !stats.bridge {
            errors.push(ShipComponentsValidation::NoBridge);
        }

        if stats.power_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedPower { amount: -stats.power_balance });
        }

        if stats.crew_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedCrew { amount: -stats.crew_balance });
        }

        if stats.engineer_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedEngineer { amount: -stats.engineer_balance });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug,Clone)]
pub struct WeaponState {
    pub recharge: f32,
}

impl WeaponState {
    pub fn new() -> Self {
        WeaponState { recharge: 0.0 }
    }
}

#[derive(Debug,Clone)]
pub struct ShipInstance {
    pub id: ShipInstanceId,
    pub spec: ShipSpec,
    pub armor_damage: HashSet<u32>,
    pub component_damage: HashMap<u32, u32>,
    pub weapons_state: HashMap<ComponentId, Vec<WeaponState>>,
}

impl ShipInstance {
    pub fn new(components: &Components, id: ShipInstanceId, spec: ShipSpec) -> Self {
        let mut weapons_state = HashMap::new();

        for weapon_id in spec.find_weapons(components) {
            let amount = *spec.amount(&weapon_id).unwrap();
            let mut vec = vec![];
            for _ in 0..amount {
                vec.push(WeaponState::new());
            }
            weapons_state.insert(weapon_id.clone(), vec);
        }

        ShipInstance { id, spec, armor_damage: Default::default(), component_damage: Default::default(), weapons_state: weapons_state }
    }

    pub fn get_weapon_state(&mut self, id: &ComponentId, index: u32) -> &mut WeaponState {
        self.weapons_state.get_mut(id).unwrap().get_mut(index as usize).unwrap()
    }
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

#[derive(Debug, Clone)]
pub struct ShipStats {
    pub bridge: bool,
    pub total_weight: u32,
    pub total_width: u32,
    pub power_balance: f32,
    pub crew_balance: f32,
    pub engineer_balance: f32,
    pub thrust: f32,
    pub speed: f32,
}


fn compute_weight(components: &Components, ship_components: &HashMap<ComponentId, u32>, armor: &Armor) -> u32 {
    let sum: f32 = ship_components.iter().map(|(component_id, amount)| {
        components.get(component_id).weight * *amount as f32
    }).sum();

    let armor_sum = (armor.width * armor.height * 10) as f32;

    round_weight(sum + armor_sum)
}

fn round_width(value: f32) -> u32 {
    value.ceil() as u32
}

fn round_weight(value: f32) -> u32 {
    value.ceil() as u32
}
