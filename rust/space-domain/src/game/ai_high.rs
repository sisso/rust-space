use crate::game::wares::{WareId, WareAmount};
use crate::game::objects::ObjId;
use std::collections::HashMap;
use specs::{World, Component, VecStorage, WorldExt, DenseVecStorage, Builder, System, ReadStorage, Entities, Read, LazyUpdate, WriteStorage};

///
/// AI components responsible to manage high level management.
///
/// Each fleet, station, or player publish information that is used by high commend to publish
/// task to be executed.
///
///

#[derive(Clone,Copy,Debug)]
enum Role {
    Miner,
    Cargo,
    Factory,
}

#[derive(Clone,Debug)]
enum Info {
    Idle,
    RequestResource { ware: WareAmount },
    OfferResource { ware: WareAmount },
}

#[derive(Clone,Debug)]
struct  ObjOrder {
    id: ObjId,
    order: Order,
}

#[derive(Clone,Debug)]
enum Order {
    Mine,
    Deliver { to: ObjId, ware: WareAmount },
    Pickup { to: ObjId, ware: WareAmount },
}

impl Component for ObjId {
    type Storage = VecStorage<Self>;
}

impl Component for Role {
    type Storage = DenseVecStorage<Self>;
}

impl Component for Order {
    type Storage = DenseVecStorage<Self>;
}

impl Component for Info {
    type Storage = DenseVecStorage<Self>;
}

struct AssignMinersSystem {
}

impl AssignMinersSystem {
    pub fn new() -> Self {
        AssignMinersSystem { }
    }
}

impl<'a> System<'a> for AssignMinersSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, ObjId>,
        ReadStorage<'a, Role>,
        ReadStorage<'a, Info>,
        WriteStorage<'a, Order>,
    );

    fn run(&mut self, (entities, ids, roles, infos, mut orders): Self::SystemData) {
        use specs::Join;
        for (e, id, role, info) in (&entities, &ids, &roles, &infos).join() {
            match (role, info) {
                (Role::Miner, Info::Idle) => {},
                _ => continue,
            }

            if orders.get(e).is_none() {
                orders.insert(e, Order::Mine);
            }
        }
    }
}

struct DeliverResourceSystem {
    delivers: Vec<ObjId>
}

impl DeliverResourceSystem {
    pub fn new() -> Self {
        DeliverResourceSystem { delivers: vec![] }
    }
}

impl<'a> System<'a> for DeliverResourceSystem {
    type SystemData = (
        ReadStorage<'a, ObjId>,
        ReadStorage<'a, Role>,
        ReadStorage<'a, Info>,
    );

    fn run(&mut self, (ids, roles, infos): Self::SystemData) {
        use specs::Join;

        // collect all request by ware and offers by ware
//        for info in &infos {
//
//        }

        // for each match, assign orders
    }
}

struct CollectOrdersSystem {
    orders: Vec<ObjOrder>,
}

impl CollectOrdersSystem {
    pub fn new() -> Self {
        CollectOrdersSystem { orders: vec![] }
    }

    pub fn take_orders(&mut self) -> Vec<ObjOrder> {
        std::mem::replace(&mut self.orders, vec![])
    }
}

impl<'a> System<'a> for CollectOrdersSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, ObjId>,
        ReadStorage<'a, Order>,
    );

    fn run(&mut self, (entities, ids, orders): Self::SystemData) {
        use specs::Join;

        for (e, id, order) in (&entities, &ids, &orders).join() {
            self.orders.push(ObjOrder {
                id: *id,
                order: order.clone(),
            });

            entities.delete(e);
        }
    }
}

struct AiHigh {
    world: World,
    system_assign_miner: AssignMinersSystem,
    collect_orders: CollectOrdersSystem,
}

impl AiHigh {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<ObjId>();
        world.register::<Role>();
        world.register::<Order>();
        world.register::<Info>();

        AiHigh {
            world,
            system_assign_miner: AssignMinersSystem::new(),
            collect_orders: CollectOrdersSystem::new(),
        }
    }

    pub fn set_info(&mut self, id: ObjId, role: Role, info: Info) {
        self.world.create_entity()
            .with(id)
            .with(role)
            .with(info)
            .build();
    }

    pub fn execute(&mut self) {
        use specs::RunNow;
        self.system_assign_miner.run_now(&self.world);
        self.collect_orders.run_now(&self.world);
    }

    pub fn take_orders(&mut self) -> Vec<ObjOrder> {
        self.collect_orders.take_orders()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::objects::{Obj, ObjId};

    #[test]
    pub fn test_ai_high_should_allocate_idle_miners_to_mine() {
        let mut ai = AiHigh::new();
        let miner_id  = ObjId(0);
        ai.set_info(miner_id, Role::Miner, Info::Idle);
        ai.execute();
        let orders = ai.take_orders();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders.get(0).unwrap().id, miner_id);
        match &orders.get(0).unwrap().order {
            Order::Mine => {},
            other => {
                panic!("receive unexpected order type");
            }
        }
    }

    #[test]
    pub fn test_ai_high_should_send_miner_to_deliver_ore() {
        let mut ai = AiHigh::new();
        let ore_id = WareId(0);
        let miner_id  = ObjId(0);
        let ore_processor_id = ObjId(1);
        ai.set_info(miner_id, Role::Miner, Info::OfferResource { ware: WareAmount(ore_id, 1.0) });

        // ignore if offer if there is no request
        ai.execute();
        let orders = ai.take_orders();
        assert_eq!(orders.len(), 0);

        // request made, send deliver
        ai.set_info(ore_processor_id, Role::Factory, Info::RequestResource { ware: WareAmount(ore_id, 10.0) });
        ai.execute();
        let orders = ai.take_orders();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders.get(0).unwrap().id, miner_id);
        match &orders.get(0).unwrap().order {
            Order::Deliver { to, ware } => {
                assert_eq!(*to,  ore_processor_id);
                assert_eq!(ware.0, ore_id);
                assert_eq!(ware.1, 1.0);
            },
            other => {
                panic!("receive unexpected order type");
            }
        }
    }
}
