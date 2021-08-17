#![no_std]
#![feature(const_generics)]

#![cfg_attr(all(target_os="nanos", test), no_main)]
#![cfg_attr(target_os="nanos", feature(custom_test_frameworks))]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(target_os="nanos", test_runner(nanos_sdk::sdk_test_runner))]

#[cfg(all(target_os = "nanos", test))]
#[no_mangle]
extern "C" fn sample_main() {
    use nanos_sdk::exit_app;
    test_main();
    exit_app(0);
}



use core::panic::PanicInfo;
/// In case of runtime problems, return an internal error and exit the app
#[cfg(all(target_os="nanos", test))]
#[inline]
#[cfg_attr(all(target_os="nanos", test), panic_handler)]
pub fn exiting_panic(info: &PanicInfo) -> ! {
    //let mut comm = io::Comm::new();
    //comm.reply(io::StatusWords::Panic);
    write!(DBG, "Panicking: {:?}\n", info);
    nanos_sdk::exit_app(1)
}

/// Custom type used to implement tests
#[cfg(all(target_os = "nanos", test))]
use nanos_sdk::TestType;

#[cfg(target_os="nanos")]
#[macro_export]
macro_rules! def_parsers {
    {$stateset_name:ident $parser_tags:ident { $($name:ident = $parser:expr; )+}} =>
    {
        enum StateSet<$($name),+>{
            NoState,
            $($name($name)),+
        }
        fn state_init<$($name),+>($(_: $name),+) -> StateSet<$($name),+> { StateSet::NoState }

        pub type RX<'a, R> = Result<(R, &'a [u8]), (Option<OOB>, &'a [u8] )>;

        #[derive(Copy, Clone)]
        pub enum $parser_tags {
            Reset,
            $($name),+
        }

        use arrayvec::ArrayVec;

        pub fn $stateset_name() -> impl for<'a> FnMut($parser_tags, &'a [u8]) -> RX<'a, ArrayVec<u8, 260> > {
            $(#[allow(non_snake_case)] let $name = $parser;)+
            let mut state_enum = state_init($($name.init_method()),+);

            move |selector, chunk| {
                match selector {
                    $parser_tags::Reset => {
                        state_enum = StateSet::NoState;
                        Err((None, chunk))
                    }
                    $($parser_tags::$name => {
                        match state_enum {
                            StateSet::$name(_) => { }
                            _ => state_enum = StateSet::$name($name.init_method())
                        }
                        match state_enum {
                            StateSet::$name(ref mut a) => {
                                $name.parse(a, chunk)
                            }
                            _ => { panic!("Unreachable"); }
                        }
                    })+
                }
            }
        }
    }
}

#[cfg(not(target_os="nanos"))]
macro_rules! def_parsers {
    {$stateset_name:ident $parser_tags:ident { $($name:ident = $parser:expr; )+}} =>
    {}
}

use ledger_parser_combinators::core_parsers::{U32, Byte, DArray, Action};
use ledger_parser_combinators::forward_parser::{ForwardParser, OOB};
use ledger_parser_combinators::endianness::Endianness;

// Fiddly; this one's basically just fmap rather than the more monadic-like Action.
// Relevant because it's inconvenient with current Action to return a non-copy item like ArrayVec.
mod fa {
    use ledger_parser_combinators::core_parsers::RV;
    use ledger_parser_combinators::forward_parser::{ForwardParser, OOB};
    type RX<'a, R> = Result<(R, &'a [u8]), (Option<OOB>, &'a [u8] )>;
    type RR<'a, I> = RX<'a, <I as RV>::R>;

    pub struct FinalAction<I : RV, O, F: Fn(&I::R) -> O> {
        pub sub: I,
        pub f: F
    }
    impl<I : RV, O, F: Fn(&I::R) -> O> RV for FinalAction<I,O,F> {
        type R = O;
    }
    impl<I : RV + ForwardParser, O, F: Fn(&I::R) -> O> ForwardParser for FinalAction<I, O, F> {
        type State = I::State;
        fn init() -> Self::State { I::init() }
        fn parse<'a, 'b>(&self, state: &'b mut Self::State, chunk: &'a [u8]) -> RR<'a, Self>{
            let (ret, new_chunk) = self.sub.parse(state, chunk)?;
            Ok(((self.f)(&ret), new_chunk))
        }
    }
}

// Define the APDU-handling parsers; clustered together like this to allow us to infer a type for
// the big enum of global parser states.

#[cfg(all(target_os = "nanos", test))]
use nanos_sdk::debug_print;
#[cfg(not(all(target_os = "nanos", test)))]
fn debug_print(s: &str) {
    
}

struct DBG;
use core;
use core::fmt;
use core::fmt::Write;
use arrayvec::ArrayString;
impl core::fmt::Write for DBG {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Dunno why the copy is required, might be some pic issue as this is going straight to
        // assembly.
        let mut qq = ArrayString::<128>::new();
        qq.push_str(s);
        debug_print(qq.as_str());
        Ok(())
    }
}

def_parsers!{ mk_parsers ParserTag {
    GetAddressParser = Action {
        sub: DArray::<_,_,10>(Byte, U32::< { Endianness::Little } >),
        f: | path | {
            let mut raw_key = [0u8; 32];
            use nanos_sdk::ecc::{CurvesId};
            match nanos_sdk::ecc::bip32_derive(CurvesId::Secp256k1, &path[..], &mut raw_key) {
                Ok(_) => {
                    let mut rv = ArrayVec::<u8, 260>::new();
                    rv.try_extend_from_slice(&raw_key);
                    let mut pmpt = [ArrayString::new(), ArrayString::new()];
                    write!(pmpt[0], "Provide Public Key");
                    write!(pmpt[1], "{:?}", path);
                    (rv, Some(OOB::Prompt(pmpt)))
                }
                Err(_) => { panic!("Need to be able to reject from here; fix Action so we can"); }
            }
        }
    };
} }


#[cfg(all(target_os="nanos", test))]
mod tests {
    use ledger_parser_combinators::core_parsers::{U32, Byte, DArray, Array};
    use ledger_parser_combinators::forward_parser::{ForwardParser, OOB};
    use ledger_parser_combinators::endianness::Endianness;
    use super::TestType;
    use testmacro::test_item as test;
    use nanos_sdk::{debug_print, assert_eq_err};
    use arrayvec::ArrayString;
    use core::fmt::write;

#[test]
    fn test_byte() {
    let mut parser = Byte;
    let mut state = parser.init_method();
    let data = [0x01,0x00,0x00,0x00,0x00];
    debug_print("In byte test\n");
    let s = ArrayString::<10>::new();
    let rv = parser.parse(&mut state, &data);
    assert_eq_err!(rv.ok(), Some((1, &[0,0,0,0][..])));

    // assert_eq!(1,0);
    debug_print("Continued\n");
}

#[test]
    fn test_U32() {
    let mut parser = U32::< { Endianness::Little }>;
    let mut state = parser.init_method();
    let data = [0x01,0x00,0x00,0x00,0x00];
    debug_print("In byte test\n");
    let s = ArrayString::<10>::new();
    let rv = parser.parse(&mut state, &data);
    assert_eq_err!(rv.ok(), Some((1, &[0][..])));

    // assert_eq!(1,0);
    debug_print("Continued\n");
}

#[test]
fn test_darray_1() {
    let parser = Array::<_,4>(Byte);
    // let mut parser = DArray::<_,_,100>(U32::< { Endianness::Little } >, Byte);
    let mut state = parser.init_method();
    let data = [0x01,0x00,0x00,0x00,0x00];
    debug_print("In darray test\n");
    parser.parse(&mut state, &data);
    // assert_eq!(1,0);
    debug_print("Continued\n");
}

#[test]
fn test_darray() {
    let parser = DArray::<_,_,10>(Byte, U32::< { Endianness::Little }>);
    let mut state = parser.init_method();
    let data = [0x01,0x00,0x00,0x00,0x00];
    debug_print("In darray test\n");
    // parser.parse(&mut state, &data);
    // assert_eq!(1,0);
    debug_print("Continued\n");
}

#[test]
fn test_getaddress() {
    let mut parsers = super::mk_parsers();
    let data = [0x01,0x00,0x00,0x00,0x00];
    debug_print("In getaddress test\n");
    // assert_eq!(1,0);
    debug_print("Continued\n");
    // parsers(super::ParserTag::GetAddressParser, &data);
}
}

