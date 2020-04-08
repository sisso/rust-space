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

    pub enum InputsOffset {}
    #[derive(Copy, Clone, Debug, PartialEq)]

    pub struct Inputs<'a> {
        pub _tab: flatbuffers::Table<'a>,
    }

    impl<'a> flatbuffers::Follow<'a> for Inputs<'a> {
        type Inner = Inputs<'a>;
        #[inline]
        fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
            Self {
                _tab: flatbuffers::Table { buf: buf, loc: loc },
            }
        }
    }

    impl<'a> Inputs<'a> {
        #[inline]
        pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
            Inputs { _tab: table }
        }
        #[allow(unused_mut)]
        pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
            _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
            args: &'args InputsArgs,
        ) -> flatbuffers::WIPOffset<Inputs<'bldr>> {
            let mut builder = InputsBuilder::new(_fbb);
            builder.add_new_game(args.new_game);
            builder.finish()
        }

        pub const VT_NEW_GAME: flatbuffers::VOffsetT = 4;

        #[inline]
        pub fn new_game(&self) -> bool {
            self._tab
                .get::<bool>(Inputs::VT_NEW_GAME, Some(false))
                .unwrap()
        }
    }

    pub struct InputsArgs {
        pub new_game: bool,
    }
    impl<'a> Default for InputsArgs {
        #[inline]
        fn default() -> Self {
            InputsArgs { new_game: false }
        }
    }
    pub struct InputsBuilder<'a: 'b, 'b> {
        fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
        start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
    }
    impl<'a: 'b, 'b> InputsBuilder<'a, 'b> {
        #[inline]
        pub fn add_new_game(&mut self, new_game: bool) {
            self.fbb_
                .push_slot::<bool>(Inputs::VT_NEW_GAME, new_game, false);
        }
        #[inline]
        pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> InputsBuilder<'a, 'b> {
            let start = _fbb.start_table();
            InputsBuilder {
                fbb_: _fbb,
                start_: start,
            }
        }
        #[inline]
        pub fn finish(self) -> flatbuffers::WIPOffset<Inputs<'a>> {
            let o = self.fbb_.end_table(self.start_);
            flatbuffers::WIPOffset::new(o.value())
        }
    }
} // pub mod space_data
