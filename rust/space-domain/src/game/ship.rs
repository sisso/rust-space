pub mod damages;
pub mod ship_combat;
pub mod ship_internals;

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::ship::ship_internals::{
        Component, ComponentId, Components, ShipComponentsConfig,
    };
    use commons::hocon;

    #[test]
    fn test_read_csv_components() {
        let path = crate::game::data::get_file("ship_components.conf").unwrap();
        let components: ShipComponentsConfig =
            hocon::load_file(&path).expect("failed to load components");
        println!("{:?}", components);
    }

    #[test]
    fn test_ship_creation() {
        let mut components = Components::new();
        components.add(Component {
            id: ComponentId(0),
            weapon: None,
            thrust: 0.0,
            weight: 0,
            crew: 0.0,
            power: 0.0,
            engineer: 0.0,
            fuel_consume: 0.0,
            size: 0,
            fuel_hold: 0.0,
            bridge_power: 0.0,
        });

        // create a ship specs
        // let specs = ShipSpec::new();
        // add components
        // check stats
    }
}
