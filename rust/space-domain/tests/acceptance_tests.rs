extern crate space_domain;

//use space_domain::game::Game;
//use space_domain::game::objects::ObjId;
//use space_domain::utils::{Seconds, V2, DeltaTime, TotalTime};
//use space_domain::game::sectors::SectorId;
//
//struct BasicScenary {
//    game: Game
//}
//
//impl BasicScenary {
//    pub fn new() -> Self {
//        let mut game = Game::new();
////        space_domain::local_game::init_new_game(&mut game);
//        BasicScenary {
//            game
//        }
//    }
//
//    pub fn get_ship(&self) -> ObjId {
////        let mut ships: Vec<ObjId> = self.game.objects.list().filter_map(|obj| {
////            self.game.locations.get_speed(&obj.id).map(|_| obj.id.clone())
////        }).collect();
////
////        ships.remove(0)
//        unimplemented!();
//    }
//}
//
//#[test]
//fn test_jump_should_warp_into_correct_position() {
//    let mut scenary = BasicScenary::new();
//    let obj_id = scenary.get_ship();
//    for i in 0..100 {
//        let total_time = TotalTime(i as f64);
//        scenary.game.tick(total_time, DeltaTime(0.5));
//
//        let location = scenary.game.locations.get_location(&obj_id);
//
//        match location {
//            Some(Location::Space { sector_id, pos }) => {
//                if *sector_id == SectorId(1) {
//                    // when ship jump into sector 2, it should be into this position
//                    let distance = V2::new(5.0, -5.0).sub(pos).length();
//                    assert!(distance < 0.1, format!("unexpected distance {:?}, ship position {:?}", distance, pos));
//
//                    // we are good
//                    return;
//                }
//            },
//            _ => {}
//        }
//    }
//
//    assert!(false, "ship not jump into time");
//}
