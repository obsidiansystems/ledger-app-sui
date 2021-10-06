#![cfg_attr(target_os = "nanos", no_std)]
#![cfg_attr(target_os = "nanos", no_main)]

#[cfg(not(target_os = "nanos"))]
fn main() {}

#[cfg(target_os = "nanos")]
use crate::crypto_helpers::*;
#[cfg(target_os = "nanos")]
use crate::implementation::*;
#[cfg(target_os = "nanos")]
use crate::interface::*;
#[cfg(target_os = "nanos")]
mod utils;

#[cfg(target_os = "nanos")]
use core::fmt::Write;
#[cfg(target_os = "nanos")]
use core::str::from_utf8;
#[cfg(target_os = "nanos")]
use nanos_sdk::buttons::ButtonEvent;
#[cfg(target_os = "nanos")]
use nanos_sdk::io;
#[cfg(target_os = "nanos")]
use nanos_ui::ui;
#[cfg(target_os = "nanos")]
use rust_app::DBG;

#[cfg(target_os = "nanos")]
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

/// Display public key in two separate
/// message scrollers
#[cfg(target_os = "nanos")]
fn show_pubkey() {
    let pubkey = get_pubkey(&BIP32_PATH);
    match pubkey {
        Ok(pk) => {
            {
                let hex0 = utils::to_hex(&pk.W[1..33]).unwrap();
                let m = from_utf8(&hex0).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
            {
                let hex1 = utils::to_hex(&pk.W[33..65]).unwrap();
                let m = from_utf8(&hex1).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
        }
        Err(_) => ui::popup("Error"),
    }
}

/// Basic nested menu. Will be subject
/// to simplifications in the future.
#[cfg(target_os = "nanos")]
#[allow(clippy::needless_borrow)]
fn menu_example() {
    loop {
        match ui::Menu::new(&[&"PubKey", &"Infos", &"Back", &"Exit App"]).show() {
            0 => show_pubkey(),
            1 => loop {
                match ui::Menu::new(&[&"Copyright", &"Authors", &"Back"]).show() {
                    0 => ui::popup("2020 Ledger"),
                    1 => ui::popup("???"),
                    _ => break,
                }
            },
            2 => return,
            3 => nanos_sdk::exit_app(2),
            _ => (),
        }
    }
}

#[cfg(target_os = "nanos")]
use ledger_parser_combinators::interp_parser::OOB;
#[cfg(target_os = "nanos")]
use rust_app::*;

#[cfg(target_os = "nanos")]
#[cfg(not(test))]
#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    // let mut states = parser_states!();
    // let mut parsers = mk_parsers();
    let mut states = ParsersState::NoState;

    use core::mem::size_of_val;
    write!(DBG, "State struct uses {} bytes\n", size_of_val(&states)).unwrap_or(());
    // with_parser_state!(parsers);

    loop {
        // Draw some 'welcome' screen
        ui::SingleMessage::new("W e l c o m e").show();

        // Wait for either a specific button push to exit the app
        // or an APDU command
        match comm.next_event() {
            io::Event::Button(ButtonEvent::RightButtonRelease) => nanos_sdk::exit_app(0),
            io::Event::Command(ins) => match handle_apdu(&mut comm, ins, &mut states) {
                Ok(()) => comm.reply_ok(),
                Err(sw) => comm.reply(sw),
            },
            _ => (),
        }
    }
}

#[repr(u8)]
enum Ins {
    GetPubkey,
    Sign,
    Menu,
    ShowPrivateKey,
    Exit,
}

impl From<u8> for Ins {
    fn from(ins: u8) -> Ins {
        match ins {
            2 => Ins::GetPubkey,
            3 => Ins::Sign,
            4 => Ins::Menu,
            0xfe => Ins::ShowPrivateKey,
            0xff => Ins::Exit,
            _ => panic!(),
        }
    }
}

#[cfg(target_os = "nanos")]
use arrayvec::ArrayVec;
#[cfg(target_os = "nanos")]
use nanos_sdk::debug_print;
#[cfg(target_os = "nanos")]
use nanos_sdk::io::Reply;

#[cfg(target_os = "nanos")]
use ledger_parser_combinators::interp_parser::InterpParser;
#[cfg(target_os = "nanos")]
fn run_parser_apdu<P: InterpParser<A, Returning = ArrayVec<u8, 260>>, A>(
    states: &mut ParsersState,
    get_state: fn(&mut ParsersState) -> &mut <P as InterpParser<A>>::State,
    parser: &P,
    comm: &mut io::Comm,
) -> Result<(), Reply> {
    let cursor = comm.get_data()?;

    loop {
        write!(DBG, "Parsing APDU input: {:?}\n", cursor);
        let parse_rv = <P as InterpParser<A>>::parse(parser, get_state(states), cursor);
        write!(DBG, "Parser result: {:?}\n", parse_rv);
        match parse_rv {
            // Explicit rejection; reset the parser. Possibly send error message to host?
            Err((Some(OOB::Reject), _)) => {
                *states = ParsersState::NoState;
                break Err(io::StatusWords::Unknown.into());
            }
            // Deliberately no catch-all on the Err((Some case; we'll get error messages if we
            // add to OOB's out-of-band actions and forget to implement them.
            //
            // Finished the chunk with no further actions pending, but not done.
            Err((None, [])) => break Ok(()),
            // Didn't consume the whole chunk; reset and error message.
            Err((None, _)) => {
                *states = ParsersState::NoState;
                break Err(io::StatusWords::Unknown.into());
            }
            // Consumed the whole chunk and parser finished; send response.
            Ok((rv, [])) => {
                comm.append(&rv[..]);
                // Parse finished; reset.
                *states = ParsersState::NoState;
                break Ok(());
            }
            // Parse ended before the chunk did; reset.
            Ok((_, _)) => {
                *states = ParsersState::NoState;
                break Err(io::StatusWords::Unknown.into());
            }
        }
    }
}

#[cfg(target_os = "nanos")]
// fn handle_apdu<P: for<'a> FnMut(ParserTag, &'a [u8]) -> RX<'a, ArrayVec<u8, 260> > >(comm: &mut io::Comm, ins: Ins, parser: &mut P) -> Result<(), Reply> {
fn handle_apdu(comm: &mut io::Comm, ins: Ins, parser: &mut ParsersState) -> Result<(), Reply> {
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    match ins {
        Ins::GetPubkey => {
            run_parser_apdu::<_, Bip32Key>(parser, get_get_address_state, &GET_ADDRESS_IMPL, comm)?
        }
        Ins::Sign => {
            run_parser_apdu::<_, SignParameters>(parser, get_sign_state, &SIGN_IMPL, comm)?
        }

        Ins::Menu => menu_example(),
        Ins::ShowPrivateKey => comm.append(&bip32_derive_secp256k1(&BIP32_PATH)?),
        Ins::Exit => nanos_sdk::exit_app(0),
        // _ => nanos_sdk::exit_app(0)
    }
    Ok(())
}
