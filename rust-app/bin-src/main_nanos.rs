use rust_app::implementation::*;
use rust_app::interface::*;

use ledger_parser_combinators::interp_parser::OOB;
use ledger_prompts_ui::RootMenu;

use nanos_sdk::io;
nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

use rust_app::*;

#[cfg(not(test))]
#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();
    let mut states = ParsersState::NoState;

    let mut idle_menu = RootMenu::new([ concat!("Rust App ", env!("CARGO_PKG_VERSION")), "Exit" ]);
    let mut busy_menu = RootMenu::new([ "Working...", "Cancel" ]);

    info!("Rust App {}", env!("CARGO_PKG_VERSION"));
    info!("State sizes\ncomm: {}\nstates: {}"
          , core::mem::size_of::<io::Comm>()
          , core::mem::size_of::<ParsersState>());

    let // Draw some 'welcome' screen
        menu = |states : &ParsersState, idle : & mut RootMenu<2>, busy : & mut RootMenu<2>| {
            match states {
                ParsersState::NoState => idle.show(),
                _ => busy.show(),
            }
        };

    menu(&states, & mut idle_menu, & mut busy_menu);
    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        match comm.next_event() {
            io::Event::Command(ins) => {
                trace!("Command received");
                match handle_apdu(&mut comm, ins, &mut states) {
                    Ok(()) => {
                        trace!("APDU accepted; sending response");
                        comm.reply_ok();
                        trace!("Replied");
                    }
                    Err(sw) => comm.reply(sw),
                };
                menu(&states, & mut idle_menu, & mut busy_menu);
                trace!("Command done");
            }
            io::Event::Button(btn) => {
                trace!("Button received");
                match states {
                    ParsersState::NoState => {match idle_menu.update(btn) {
                        Some(1) => { info!("Exiting app at user direction via root menu"); nanos_sdk::exit_app(0) },
                        _ => (),
                    } }
                    _ => { match busy_menu.update(btn) {
                        Some(1) => { info!("Resetting at user direction via busy menu"); reset_parsers_state(&mut states) }
                        _ => (),
                    } }
                };
                menu(&states, & mut idle_menu, & mut busy_menu);
                trace!("Button done");
            }
            io::Event::Ticker => {
                //trace!("Ignoring ticker event");
            },
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
enum Ins {
    GetVersion,
    GetPubkey,
    Sign,
    GetVersionStr,
    Exit
}

impl From<u8> for Ins {
    fn from(ins: u8) -> Ins {
        match ins {
            0 => Ins::GetVersion,
            2 => Ins::GetPubkey,
            3 => Ins::Sign,
            0xfe => Ins::GetVersionStr,
            0xff => Ins::Exit,
            _ => panic!(),
        }
    }
}

use arrayvec::ArrayVec;
use nanos_sdk::io::Reply;

use ledger_parser_combinators::interp_parser::{ParserCommon, InterpParser};
fn run_parser_apdu<P: InterpParser<A, Returning = ArrayVec<u8, 128>>, A>(
    states: &mut ParsersState,
    get_state: fn(&mut ParsersState) -> &mut <P as ParserCommon<A>>::State,
    parser: &P,
    comm: &mut io::Comm,
) -> Result<(), Reply> {
    let cursor = comm.get_data()?;

    loop {
        trace!("Parsing APDU input: {:?}\n", cursor);
        let mut parse_destination = None;
        let parse_rv = <P as InterpParser<A>>::parse(parser, get_state(states), cursor, &mut parse_destination);
        trace!("Parser result: {:?}\n", parse_rv);
        match parse_rv {
            // Explicit rejection; reset the parser. Possibly send error message to host?
            Err((Some(OOB::Reject), _)) => {
                reset_parsers_state(states);
                break Err(io::StatusWords::Unknown.into());
            }
            // Deliberately no catch-all on the Err((Some case; we'll get error messages if we
            // add to OOB's out-of-band actions and forget to implement them.
            //
            // Finished the chunk with no further actions pending, but not done.
            Err((None, [])) => { trace!("Parser needs more; continuing"); break Ok(()) }
            // Didn't consume the whole chunk; reset and error message.
            Err((None, _)) => {
                reset_parsers_state(states);
                break Err(io::StatusWords::Unknown.into());
            }
            // Consumed the whole chunk and parser finished; send response.
            Ok([]) => {
                trace!("Parser finished, resetting state\n");
                match parse_destination.as_ref() {
                    Some(rv) => comm.append(&rv[..]),
                    None => break Err(io::StatusWords::Unknown.into()),
                }
                // Parse finished; reset.
                reset_parsers_state(states);
                break Ok(());
            }
            // Parse ended before the chunk did; reset.
            Ok(_) => {
                reset_parsers_state(states);
                break Err(io::StatusWords::Unknown.into());
            }
        }
    }
}

#[inline(never)]
fn handle_apdu(comm: &mut io::Comm, ins: Ins, parser: &mut ParsersState) -> Result<(), Reply> {
    info!("entering handle_apdu with command {:?}", ins);
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    match ins {
        Ins::GetVersion => {
            comm.append(&[env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(), env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(), env!("CARGO_PKG_VERSION_PATCH").parse().unwrap()]);
            comm.append(b"rust app");
        }
        Ins::GetPubkey => {
            run_parser_apdu::<_, Bip32Key>(parser, get_get_address_state, &GET_ADDRESS_IMPL, comm)?
        }
        Ins::Sign => {
            run_parser_apdu::<_, SignParameters>(parser, get_sign_state, &SIGN_IMPL, comm)?
        }
        Ins::GetVersionStr => {
            comm.append(concat!("Rust App ", env!("CARGO_PKG_VERSION")).as_ref());
        }
        Ins::Exit => nanos_sdk::exit_app(0),
    }
    Ok(())
}
