use std::collections::{HashMap, HashSet};

use crate::utils::Speed;


#[derive(Clone, Debug)]
pub struct Armor {
    pub width: u32,
    pub height: u32,
}

impl Armor {
    pub fn new(width: u32, height: u32) -> Self {
        Armor { width, height }
    }

    pub fn weight(&self) -> f32 {
        self.width as f32 * self.height as f32 * 0.5
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Damage(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Amount(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Hp(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ArmorIndex(pub u32);

// TODO: width in meters? 10 meters per width? normal ship 19?
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Width(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Tons(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TeamId(pub u32);

#[derive(Clone, Debug)]
pub enum WeaponDamageType {
    Explosive,
    Penetration,
}

#[derive(Clone, Debug)]
pub struct Weapon {
    pub damage: Damage,
    pub reload: f32,
    pub rounds: u32,
    pub damage_type: WeaponDamageType,
}

#[derive(Clone, Debug)]
pub struct Component {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub weapon: Option<Weapon>,
    pub thrust: f32,
    pub weight: u32,
    pub crew_require: f32,
    pub crew_provide: f32,
    pub power_require: f32,
    pub power_generate: f32,
    pub engineer_provide: f32,
    pub engineer_require: f32,
    pub fuel_consume: f32,
    pub width: u32,
    pub fuel_hold: f32,
}

impl Component {
    pub fn new(id: ComponentId, component_type: ComponentType) -> Self {
        Component {
            id,
            component_type,
            weapon: None,
            thrust: 0.0,
            weight: 0,
            crew_require: 0.0,
            crew_provide: 0.0,
            power_require: 0.0,
            power_generate: 0.0,
            engineer_provide: 0.0,
            engineer_require: 0.0,
            fuel_consume: 0.0,
            width: 0,
            fuel_hold: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
struct ComponentItem {
    pub damage: u32,
    pub max_damage: u32,
    pub working: u32,
}

#[derive(Clone, Debug)]
pub struct ShipSpec {
    pub armor: Armor,
    pub components: HashMap<ComponentId, u32>,
    pub stats: ShipStats,
    pub component_table: ComponentTable,
}

impl ShipSpec {
    pub fn new(
        components: &Components,
        ship_components: HashMap<ComponentId, u32>,
        armor_height: u32,
    ) -> Self {
        let component_table = ComponentTable::new(components, &ship_components);
        let armor = Armor::new(component_table.total, armor_height);
        let stats =
            ShipSpec::compute_ship_stats(components, &ship_components, &armor, &HashMap::new());

        ShipSpec {
            armor: armor,
            components: ship_components,
            stats: stats,
            component_table,
        }
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
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn compute_ship_stats(
        components: &Components,
        ship_components: &HashMap<ComponentId, u32>,
        armor: &Armor,
        destroyed_components: &HashMap<ComponentId, Amount>,
    ) -> ShipStats {
        let mut has_bridge = false;

        let mut power = 0.0;
        let mut crew = 0.0;
        let mut engineer = 0.0;
        let mut thrust: f32 = 0.0;
        let mut weight: u32 = 0;
        let mut width: u32 = 0;

        for (id, amount) in ship_components {
            let component = components.get(id);
            let destroyed_amount = destroyed_components.get(id).unwrap_or(&Amount(0));

            weight += component.weight * amount;
            width += component.width * amount;

            if destroyed_amount.0 >= *amount {
                // all components from this category were destroyed
                continue;
            }

            let active_amount = *amount - destroyed_amount.0;
            let active_amount_f32 = active_amount as f32;

            if let ComponentType::Bridge = component.component_type {
                has_bridge = true;
            }

            power += component.power_generate * active_amount_f32
                - component.power_require * active_amount_f32;
            crew += component.crew_provide * active_amount_f32
                - component.crew_require * active_amount_f32;
            engineer += component.engineer_provide * active_amount_f32
                - component.engineer_require * active_amount_f32;
            thrust += component.thrust * active_amount_f32;
        }

        let armor_weight = armor.width * armor.height * 10;
        weight += armor_weight;

        ShipStats {
            bridge: has_bridge,
            total_weight: weight,
            total_width: width,
            power_balance: power,
            crew_balance: crew,
            engineer_balance: engineer,
            thrust: thrust,
            speed: Speed(10.0 * thrust / (weight as f32)),
        }
    }

    pub fn is_valid(&self) -> Result<(), Vec<ShipComponentsValidation>> {
        let stats = &self.stats;

        let mut errors = vec![];

        if !stats.bridge {
            errors.push(ShipComponentsValidation::NoBridge);
        }

        if stats.power_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedPower {
                amount: -stats.power_balance,
            });
        }

        if stats.crew_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedCrew {
                amount: -stats.crew_balance,
            });
        }

        if stats.engineer_balance < 0.0 {
            errors.push(ShipComponentsValidation::NeedEngineer {
                amount: -stats.engineer_balance,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn map_components<'a>(&self, components: &'a Components) -> Vec<(&'a Component, Amount)> {
        self.components
            .iter()
            .map(|(id, amount)| {
                let component = components.get(id);
                (component, Amount(*amount))
            })
            .collect()
    }

    pub fn get_hull_hp(&self, components: &Components) -> Hp {
        let total = self
            .map_components(components)
            .iter()
            .map(|(component, amount)| component.width * amount.0)
            .sum();

        Hp(total)
    }
}

#[derive(Debug, Clone)]
pub struct WeaponState {
    pub recharge: f32,
}

impl WeaponState {
    pub fn new() -> Self {
        WeaponState { recharge: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct ComponentTable {
    pub total: u32,
    pub sequence: Vec<(ComponentId, u32)>,
}

impl ComponentTable {
    pub fn new(components: &Components, amounts: &HashMap<ComponentId, u32>) -> Self {
        let sequence: Vec<(ComponentId, u32)> = amounts
            .iter()
            .map(|(id, amount)| {
                let component = components.get(id);
                let comp_width = component.width * *amount;
                (component.id, comp_width)
            })
            .collect();

        let total: u32 = sequence.iter().map(|(_, i)| i).sum();

        ComponentTable {
            total: total,
            sequence: sequence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShipInstance {
    pub id: ShipInstanceId,
    pub spec: ShipSpec,
    /// instance ship stats considering current damaged components
    pub current_stats: ShipStats,
    pub armor_damage: HashSet<ArmorIndex>,
    pub component_damage: HashMap<ComponentId, Damage>,
    pub component_destroyed: HashMap<ComponentId, Amount>,
    pub weapons_state: HashMap<ComponentId, Vec<WeaponState>>,
    pub wreck: bool,
    pub team_id: TeamId,
}

impl ShipInstance {
    pub fn new(
        components: &Components,
        id: ShipInstanceId,
        spec: ShipSpec,
        team_id: TeamId,
    ) -> Self {
        let mut weapons_state = HashMap::new();

        for weapon_id in spec.find_weapons(components) {
            let amount = *spec.amount(&weapon_id).unwrap();
            let mut vec = vec![];
            for _ in 0..amount {
                vec.push(WeaponState::new());
            }
            weapons_state.insert(weapon_id.clone(), vec);
        }

        let stats = spec.stats.clone();

        ShipInstance {
            id,
            spec,
            current_stats: stats,
            armor_damage: Default::default(),
            component_damage: Default::default(),
            component_destroyed: Default::default(),
            weapons_state,
            wreck: false,
            team_id,
        }
    }

    pub fn get_weapon_state(&mut self, id: &ComponentId, index: u32) -> &mut WeaponState {
        self.weapons_state
            .get_mut(id)
            .unwrap()
            .get_mut(index as usize)
            .unwrap()
    }

    pub fn update_stats(&mut self, components: &Components) {
        let new_stats = ShipSpec::compute_ship_stats(
            components,
            &self.spec.components,
            &self.spec.armor,
            &self.component_destroyed,
        );
        self.current_stats = new_stats;
    }

    pub fn get_total_hull_damage(&self) -> Damage {
        let total = self
            .component_damage
            .iter()
            .map(|(_, damage)| damage.0)
            .sum();

        Damage(total)
    }
}

pub struct Components {
    index: HashMap<ComponentId, Component>,
}

impl Components {
    pub fn new() -> Self {
        Components {
            index: Default::default(),
        }
    }

    //    pub fn next_id(&mut self) -> ComponentId {
    //        ComponentId(self.next_id.next())
    //    }

    pub fn add(&mut self, component: Component) {
        if self.index.contains_key(&component.id) {
            panic!();
        }

        log::info!("components - {:?} added {:?}", component.id, component);
        self.index.insert(component.id, component);
    }

    pub fn get(&self, component_id: &ComponentId) -> &Component {
        self.index.get(component_id).unwrap()
    }
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
    pub speed: Speed,
}

fn round_width(value: f32) -> u32 {
    value.ceil() as u32
}

fn round_weight(value: f32) -> u32 {
    value.ceil() as u32
}
