use crate::game::wares::{WareId, WareAmount};
use crate::game::objects::ObjId;
use std::mem;
use std::collections::{HashMap, HashSet};
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

        let mut to_add = vec![];

        for (e, id, role, info, _) in (&entities, &ids, &roles, &infos, !&orders).join() {
            match (role, info) {
                (Role::Miner, Info::Idle) => {},
                _ => continue,
            }

            to_add.push(e);
        }

        for e in to_add {
            orders.insert(e, Order::Mine);
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
    }

    pub fn take_orders(&mut self) -> Vec<ObjOrder> {
        use specs::RunNow;
        self.collect_orders.run_now(&self.world);
        self.collect_orders.take_orders()
    }
}

#[derive(Clone,Debug)]
struct Trader {
    obj_id: ObjId,
    amount: f32,
    can_move: bool,
}

impl Trader {
    pub fn new(obj_id: ObjId, amount: f32, can_move: bool) -> Self {
        Trader {
            obj_id,
            amount,
            can_move
        }
    }
}

#[derive(Clone,Debug)]
struct Transaction {
    ware_id: WareId,
    requests: Vec<Trader>,
    offers: Vec<Trader>,
}

impl Transaction {
    pub fn new(ware_id: WareId) -> Self {
        Transaction {
            ware_id,
            requests: Vec::new(),
            offers: Vec::new()
        }
    }
}

#[derive(Clone,Debug)]
struct HighAi2 {
    idle_miners: Vec<ObjId>,
    offers_by_ware: HashMap<WareId, Transaction>,
    orders: Vec<ObjOrder>,
}

impl HighAi2 {
    pub fn new() -> Self {
        HighAi2 {
            idle_miners: Vec::new(),
            offers_by_ware: HashMap::new(),
            orders: Vec::new()
        }
    }

    pub fn set_info(&mut self, id: ObjId, role: Role, info: Info) {
        let can_move = match role {
            Role::Factory => false,
            Role::Miner | Role::Cargo => true,
        };

        match (role, info) {
            (Role::Miner, Info::Idle) => {
                self.idle_miners.push(id);
            },
            (_, Info::OfferResource { ware }) => {
                self.add_trade(ware.0, can_move, true, id, ware.1);
            },
            (_, Info::RequestResource { ware }) => {
                self.add_trade(ware.0, can_move, false, id, ware.1);
            },
            other =>{
                panic!(format!("unexpected {:?}", other));
            }
        }
    }

    pub fn execute(&mut self) {
        // put lazy miners to work
        for obj_id in mem::replace(&mut self.idle_miners, Vec::new()) {
            self.orders.push(ObjOrder {
                id: obj_id,
                order: Order::Mine
            });
        }

        // fulfill the requests
        let mut removes = vec![];
        for (ware_id, trade) in &mut self.offers_by_ware.iter_mut() {
            let complete = HighAi2::find_orders_from_trade(trade, &mut self.orders);
            if complete {
                removes.push(*ware_id);
            }
        }

        for ware_id in removes {
            self.offers_by_ware.remove(&ware_id);
        }
    }

    pub fn take_orders(&mut self) -> Vec<ObjOrder> {
        mem::replace(&mut self.orders, Vec::new())
    }

    fn add_trade(&mut self, ware_id: WareId, can_move: bool, is_offer: bool, obj_id: ObjId, amount: f32) {
        let add = |trade: &mut Transaction| {
            let trader = Trader::new(obj_id, amount, can_move);

            if is_offer {
                trade.offers.push(trader);
            } else {
                trade.requests.push(trader);
            }
        };

        self.offers_by_ware.entry(ware_id)
            .and_modify(add).or_insert_with(|| {
                let mut trade = Transaction::new(ware_id);
                add(&mut trade);
                trade
            });
    }

    fn find_orders_from_trade(transaction: &mut Transaction, orders: &mut Vec<ObjOrder>) -> bool {
        println!("{:?} {:?}", transaction, orders);

        // find a movable offer
        let offer = transaction.offers.iter()
            .find(|trader| trader.can_move);

        // find a static request
        let request = transaction.requests.iter()
            .find(|trader| !trader.can_move);

        if offer.is_none() || request.is_none() {
            return false;
        }

        let offer = offer.unwrap();
        let request = request.unwrap();
        let amount = offer.amount.min(request.amount);

        orders.push(ObjOrder {
            id: offer.obj_id,
            order: Order::Deliver {
                to: request.obj_id,
                ware: WareAmount(transaction.ware_id, amount)
            }
        });

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::objects::{Obj, ObjId};

    #[test]
    pub fn test_ai_high_should_allocate_idle_miners_to_mine() {
        let mut ai = HighAi2::new();
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
        let mut ai = HighAi2::new();
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
