// automatically generated by the FlatBuffers compiler, do not modify

use std::cmp::Ordering;
use std::mem;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(unused_imports, dead_code)]
pub mod space_data {

    use std::cmp::Ordering;
    use std::mem;

    extern crate flatbuffers;
    use self::flatbuffers::EndianScalar;

    #[allow(non_camel_case_types)]
    #[repr(i16)]
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum EntityKind {
        Fleet = 0,
        Asteroid = 1,
        Station = 2,
        Jump = 3,
    }

    const ENUM_MIN_ENTITY_KIND: i16 = 0;
    const ENUM_MAX_ENTITY_KIND: i16 = 3;

    impl<'a> flatbuffers::Follow<'a> for EntityKind {
        type Inner = Self;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::read_scalar_at::<Self>(buf, loc)
        }
    }

    impl flatbuffers::EndianScalar for EntityKind {
        #[inline]
        fn to_little_endian(self) -> Self {
            let n = i16::to_le(self as i16);
            let p = &n as *const i16 as *const EntityKind;
            unsafe { *p }
        }
        #[inline]
        fn from_little_endian(self) -> Self {
            let n = i16::from_le(self as i16);
            let p = &n as *const i16 as *const EntityKind;
            unsafe { *p }
        }
    }

    impl flatbuffers::Push for EntityKind {
        type Output = EntityKind;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            flatbuffers::emplace_scalar::<EntityKind>(dst, *self);
        }
    }

    #[allow(non_camel_case_types)]
    const ENUM_VALUES_ENTITY_KIND: [EntityKind; 4] = [
        EntityKind::Fleet,
        EntityKind::Asteroid,
        EntityKind::Station,
        EntityKind::Jump,
    ];

    #[allow(non_camel_case_types)]
    const ENUM_NAMES_ENTITY_KIND: [&'static str; 4] = ["Fleet", "Asteroid", "Station", "Jump"];

    pub fn enum_name_entity_kind(e: EntityKind) -> &'static str {
        let index = e as i16;
        ENUM_NAMES_ENTITY_KIND[index as usize]
    }

    // struct V2, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct V2 {
        x_: f32,
        y_: f32,
    } // pub struct V2
    impl flatbuffers::SafeSliceAccess for V2 {}
    impl<'a> flatbuffers::Follow<'a> for V2 {
        type Inner = &'a V2;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a V2>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a V2 {
        type Inner = &'a V2;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<V2>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for V2 {
        type Output = V2;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const V2 as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b V2 {
        type Output = V2;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const V2 as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl V2 {
        pub fn new<'a>(_x: f32, _y: f32) -> Self {
            V2 {
                x_: _x.to_little_endian(),
                y_: _y.to_little_endian(),
            }
        }
        pub fn x<'a>(&'a self) -> f32 {
            self.x_.from_little_endian()
        }
        pub fn y<'a>(&'a self) -> f32 {
            self.y_.from_little_endian()
        }
    }

    // struct SectorNew, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SectorNew {
        id_: u32,
    } // pub struct SectorNew
    impl flatbuffers::SafeSliceAccess for SectorNew {}
    impl<'a> flatbuffers::Follow<'a> for SectorNew {
        type Inner = &'a SectorNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a SectorNew>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a SectorNew {
        type Inner = &'a SectorNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<SectorNew>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for SectorNew {
        type Output = SectorNew;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const SectorNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b SectorNew {
        type Output = SectorNew;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const SectorNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl SectorNew {
        pub fn new<'a>(_id: u32) -> Self {
            SectorNew {
                id_: _id.to_little_endian(),
            }
        }
        pub fn id<'a>(&'a self) -> u32 {
            self.id_.from_little_endian()
        }
    }

    // struct JumpNew, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct JumpNew {
        id_: u32,
        sector_id_: u32,
        pos_: V2,
        to_sector_id_: u32,
        to_pos_: V2,
    } // pub struct JumpNew
    impl flatbuffers::SafeSliceAccess for JumpNew {}
    impl<'a> flatbuffers::Follow<'a> for JumpNew {
        type Inner = &'a JumpNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a JumpNew>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a JumpNew {
        type Inner = &'a JumpNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<JumpNew>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for JumpNew {
        type Output = JumpNew;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const JumpNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b JumpNew {
        type Output = JumpNew;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const JumpNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl JumpNew {
        pub fn new<'a>(
            _id: u32,
            _sector_id: u32,
            _pos: &'a V2,
            _to_sector_id: u32,
            _to_pos: &'a V2,
        ) -> Self {
            JumpNew {
                id_: _id.to_little_endian(),
                sector_id_: _sector_id.to_little_endian(),
                pos_: *_pos,
                to_sector_id_: _to_sector_id.to_little_endian(),
                to_pos_: *_to_pos,
            }
        }
        pub fn id<'a>(&'a self) -> u32 {
            self.id_.from_little_endian()
        }
        pub fn sector_id<'a>(&'a self) -> u32 {
            self.sector_id_.from_little_endian()
        }
        pub fn pos<'a>(&'a self) -> &'a V2 {
            &self.pos_
        }
        pub fn to_sector_id<'a>(&'a self) -> u32 {
            self.to_sector_id_.from_little_endian()
        }
        pub fn to_pos<'a>(&'a self) -> &'a V2 {
            &self.to_pos_
        }
    }

    // struct EntityNew, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EntityNew {
        id_: u32,
        pos_: V2,
        sector_id_: u32,
        kind_: EntityKind,
        padding0__: u16,
    } // pub struct EntityNew
    impl flatbuffers::SafeSliceAccess for EntityNew {}
    impl<'a> flatbuffers::Follow<'a> for EntityNew {
        type Inner = &'a EntityNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a EntityNew>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a EntityNew {
        type Inner = &'a EntityNew;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<EntityNew>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for EntityNew {
        type Output = EntityNew;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const EntityNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b EntityNew {
        type Output = EntityNew;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const EntityNew as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl EntityNew {
        pub fn new<'a>(_id: u32, _pos: &'a V2, _sector_id: u32, _kind: EntityKind) -> Self {
            EntityNew {
                id_: _id.to_little_endian(),
                pos_: *_pos,
                sector_id_: _sector_id.to_little_endian(),
                kind_: _kind.to_little_endian(),

                padding0__: 0,
            }
        }
        pub fn id<'a>(&'a self) -> u32 {
            self.id_.from_little_endian()
        }
        pub fn pos<'a>(&'a self) -> &'a V2 {
            &self.pos_
        }
        pub fn sector_id<'a>(&'a self) -> u32 {
            self.sector_id_.from_little_endian()
        }
        pub fn kind<'a>(&'a self) -> EntityKind {
            self.kind_.from_little_endian()
        }
    }

    // struct EntityMove, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EntityMove {
        id_: u32,
        pos_: V2,
    } // pub struct EntityMove
    impl flatbuffers::SafeSliceAccess for EntityMove {}
    impl<'a> flatbuffers::Follow<'a> for EntityMove {
        type Inner = &'a EntityMove;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a EntityMove>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a EntityMove {
        type Inner = &'a EntityMove;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<EntityMove>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for EntityMove {
        type Output = EntityMove;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const EntityMove as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b EntityMove {
        type Output = EntityMove;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const EntityMove as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl EntityMove {
        pub fn new<'a>(_id: u32, _pos: &'a V2) -> Self {
            EntityMove {
                id_: _id.to_little_endian(),
                pos_: *_pos,
            }
        }
        pub fn id<'a>(&'a self) -> u32 {
            self.id_.from_little_endian()
        }
        pub fn pos<'a>(&'a self) -> &'a V2 {
            &self.pos_
        }
    }

    // struct EntityJump, aligned to 4
    #[repr(C, align(4))]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EntityJump {
        id_: u32,
        sector_id_: u32,
        pos_: V2,
    } // pub struct EntityJump
    impl flatbuffers::SafeSliceAccess for EntityJump {}
    impl<'a> flatbuffers::Follow<'a> for EntityJump {
        type Inner = &'a EntityJump;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            <&'a EntityJump>::follow(buf, loc)
        }
    }
    impl<'a> flatbuffers::Follow<'a> for &'a EntityJump {
        type Inner = &'a EntityJump;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            flatbuffers::follow_cast_ref::<EntityJump>(buf, loc)
        }
    }
    impl<'b> flatbuffers::Push for EntityJump {
        type Output = EntityJump;
        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(self as *const EntityJump as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }
    impl<'b> flatbuffers::Push for &'b EntityJump {
        type Output = EntityJump;

        #[inline]
        fn push(&self, dst: &mut [u8], _rest: &[u8]) {
            let src = unsafe {
                ::std::slice::from_raw_parts(*self as *const EntityJump as *const u8, Self::size())
            };
            dst.copy_from_slice(src);
        }
    }

    impl EntityJump {
        pub fn new<'a>(_id: u32, _sector_id: u32, _pos: &'a V2) -> Self {
            EntityJump {
                id_: _id.to_little_endian(),
                sector_id_: _sector_id.to_little_endian(),
                pos_: *_pos,
            }
        }
        pub fn id<'a>(&'a self) -> u32 {
            self.id_.from_little_endian()
        }
        pub fn sector_id<'a>(&'a self) -> u32 {
            self.sector_id_.from_little_endian()
        }
        pub fn pos<'a>(&'a self) -> &'a V2 {
            &self.pos_
        }
    }

    pub enum OutputsOffset {}
    #[derive(Copy, Clone, Debug, PartialEq)]

    pub struct Outputs<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for Outputs<'a> {
        type Inner = Outputs<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf: buf, loc: loc },
            }
        }
    }

    impl<'a> Outputs<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            Outputs { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args OutputsArgs<'args>,
        ) -> flatbuffers::WIPOffset<Outputs<'bldr>> {
            let mut builder = OutputsBuilder::new(_fbb);
            if let Some(x) = args.jumps {
                builder.add_jumps(x);
            }
            if let Some(x) = args.sectors {
                builder.add_sectors(x);
            }
            if let Some(x) = args.entities_jump {
                builder.add_entities_jump(x);
            }
            if let Some(x) = args.entities_move {
                builder.add_entities_move(x);
            }
            if let Some(x) = args.entities_new {
                builder.add_entities_new(x);
            }
            builder.finish()
        }

        pub const VT_ENTITIES_NEW: flatbuffers::VOffsetT = 4;
        pub const VT_ENTITIES_MOVE: flatbuffers::VOffsetT = 6;
        pub const VT_ENTITIES_JUMP: flatbuffers::VOffsetT = 8;
        pub const VT_SECTORS: flatbuffers::VOffsetT = 10;
        pub const VT_JUMPS: flatbuffers::VOffsetT = 12;

        #[inline]
        pub fn entities_new(&self) -> Option<&'a [EntityNew]> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<EntityNew>>>(
                    Outputs::VT_ENTITIES_NEW,
                    None,
                )
                .map(|v| v.safe_slice())
        }
        #[inline]
        pub fn entities_move(&self) -> Option<&'a [EntityMove]> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<EntityMove>>>(
                    Outputs::VT_ENTITIES_MOVE,
                    None,
                )
                .map(|v| v.safe_slice())
        }
        #[inline]
        pub fn entities_jump(&self) -> Option<&'a [EntityJump]> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<EntityJump>>>(
                    Outputs::VT_ENTITIES_JUMP,
                    None,
                )
                .map(|v| v.safe_slice())
        }
        #[inline]
        pub fn sectors(&self) -> Option<&'a [SectorNew]> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<SectorNew>>>(
                    Outputs::VT_SECTORS,
                    None,
                )
                .map(|v| v.safe_slice())
        }
        #[inline]
        pub fn jumps(&self) -> Option<&'a [JumpNew]> {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<JumpNew>>>(
                    Outputs::VT_JUMPS,
                    None,
                )
                .map(|v| v.safe_slice())
        }
    }

    pub struct OutputsArgs<'a> {
        pub entities_new: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, EntityNew>>>,
        pub entities_move: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, EntityMove>>>,
        pub entities_jump: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, EntityJump>>>,
        pub sectors: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, SectorNew>>>,
        pub jumps: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, JumpNew>>>,
    }
    impl<'a> Default for OutputsArgs<'a> {
        #[inline]
        fn default() -> Self {
            OutputsArgs {
                entities_new: None,
                entities_move: None,
                entities_jump: None,
                sectors: None,
                jumps: None,
            }
        }
    }
    pub struct OutputsBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> OutputsBuilder<'a, 'b> {
        #[inline]
        pub fn add_entities_new(
            &mut self,
            entities_new: flatbuffers::WIPOffset<flatbuffers::Vector<'b, EntityNew>>,
        ) {
            self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
                Outputs::VT_ENTITIES_NEW,
                entities_new,
            );
        }
        #[inline]
        pub fn add_entities_move(
            &mut self,
            entities_move: flatbuffers::WIPOffset<flatbuffers::Vector<'b, EntityMove>>,
        ) {
            self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
                Outputs::VT_ENTITIES_MOVE,
                entities_move,
            );
        }
        #[inline]
        pub fn add_entities_jump(
            &mut self,
            entities_jump: flatbuffers::WIPOffset<flatbuffers::Vector<'b, EntityJump>>,
        ) {
            self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
                Outputs::VT_ENTITIES_JUMP,
                entities_jump,
            );
        }
        #[inline]
        pub fn add_sectors(
            &mut self,
            sectors: flatbuffers::WIPOffset<flatbuffers::Vector<'b, SectorNew>>,
        ) {
            self.fbb_
                .push_slot_always::<flatbuffers::WIPOffset<_>>(Outputs::VT_SECTORS, sectors);
        }
        #[inline]
        pub fn add_jumps(
            &mut self,
            jumps: flatbuffers::WIPOffset<flatbuffers::Vector<'b, JumpNew>>,
        ) {
            self.fbb_
                .push_slot_always::<flatbuffers::WIPOffset<_>>(Outputs::VT_JUMPS, jumps);
        }
        #[inline]
        pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> OutputsBuilder<'a, 'b> {
            let start = _fbb.start_table();
            OutputsBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<Outputs<'a>> {
            let o = self.fbb_.end_table(self.start_);
            flatbuffers::WIPOffset::new(o.value())
        }
    }

    #[inline]
    pub fn get_root_as_outputs<'a>(buf: &'a [u8]) -> Outputs<'a> {
        flatbuffers::get_root::<Outputs<'a>>(buf)
    }

    #[inline]
    pub fn get_size_prefixed_root_as_outputs<'a>(buf: &'a [u8]) -> Outputs<'a> {
        flatbuffers::get_size_prefixed_root::<Outputs<'a>>(buf)
    }

    #[inline]
    pub fn finish_outputs_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<Outputs<'a>>,
    ) {
        fbb.finish(root, None);
    }

    #[inline]
    pub fn finish_size_prefixed_outputs_buffer<'a, 'b>(
        fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        root: flatbuffers::WIPOffset<Outputs<'a>>,
    ) {
        fbb.finish_size_prefixed(root, None);
    }
} // pub mod space_data
