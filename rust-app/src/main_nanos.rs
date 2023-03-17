use crate::implementation::*;
use crate::interface::*;
use crate::menu::*;
use crate::settings::*;
use crate::test_parsers::*;

use ledger_log::{info, trace};
use ledger_parser_combinators::interp_parser::OOB;
use ledger_prompts_ui::{handle_menu_button_event, show_menu};
use nanos_sdk::io;

#[allow(dead_code)]
pub fn app_main() {
    let mut comm = io::Comm::new();
    let mut states = ParsersState::NoState;
    let mut idle_menu = IdleMenuWithSettings {
        idle_menu: IdleMenu::AppMain,
        settings: Settings::new(),
    };
    let mut busy_menu = BusyMenu::Working;

    info!("Alamgu Example {}", env!("CARGO_PKG_VERSION"));
    info!(
        "State sizes\ncomm: {}\nstates: {}",
        core::mem::size_of::<io::Comm>(),
        core::mem::size_of::<ParsersState>()
    );

    let menu = |states: &ParsersState, idle: &IdleMenuWithSettings, busy: &BusyMenu| match states {
        ParsersState::NoState => show_menu(idle),
        _ => show_menu(busy),
    };

    // Draw some 'welcome' screen
    menu(&states, &idle_menu, &busy_menu);
    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        match comm.next_event::<Ins>() {
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
                // Reset BusyMenu if we are done handling APDU
                match states {
                    ParsersState::NoState => busy_menu = BusyMenu::Working,
                    _ => {}
                };
                menu(&states, &idle_menu, &busy_menu);
                trace!("Command done");
            }
            io::Event::Button(btn) => {
                trace!("Button received");
                match states {
                    ParsersState::NoState => {
                        if let Some(DoExitApp) = handle_menu_button_event(&mut idle_menu, btn) {
                            info!("Exiting app at user direction via root menu");
                            nanos_sdk::exit_app(0)
                        }
                    }
                    _ => {
                        if let Some(DoCancel) = handle_menu_button_event(&mut busy_menu, btn) {
                            info!("Resetting at user direction via busy menu");
                            reset_parsers_state(&mut states)
                        }
                    }
                };
                menu(&states, &idle_menu, &busy_menu);
                trace!("Button done");
            }
            io::Event::Ticker => {
                //trace!("Ignoring ticker event");
            }
        }
    }
}

use arrayvec::ArrayVec;
use nanos_sdk::io::Reply;

use ledger_parser_combinators::interp_parser::{InterpParser, ParserCommon};
fn run_parser_apdu<P: InterpParser<A, Returning = ArrayVec<u8, 128>>, A>(
    states: &mut ParsersState,
    get_state: fn(&mut ParsersState) -> &mut <P as ParserCommon<A>>::State,
    parser: &P,
    comm: &mut io::Comm,
) -> Result<(), Reply> {
    let cursor = comm.get_data()?;

    trace!("Parsing APDU input: {:?}\n", cursor);
    let mut parse_destination = None;
    let parse_rv =
        <P as InterpParser<A>>::parse(parser, get_state(states), cursor, &mut parse_destination);
    trace!("Parser result: {:?}\n", parse_rv);
    match parse_rv {
        // Explicit rejection; reset the parser. Possibly send error message to host?
        Err((Some(OOB::Reject), _)) => {
            reset_parsers_state(states);
            Err(io::StatusWords::Unknown.into())
        }
        // Deliberately no catch-all on the Err((Some case; we'll get error messages if we
        // add to OOB's out-of-band actions and forget to implement them.
        //
        // Finished the chunk with no further actions pending, but not done.
        Err((None, [])) => {
            trace!("Parser needs more; continuing");
            Ok(())
        }
        // Didn't consume the whole chunk; reset and error message.
        Err((None, _)) => {
            reset_parsers_state(states);
            Err(io::StatusWords::Unknown.into())
        }
        // Consumed the whole chunk and parser finished; send response.
        Ok([]) => {
            trace!("Parser finished, resetting state\n");
            match parse_destination.as_ref() {
                Some(rv) => comm.append(&rv[..]),
                None => return Err(io::StatusWords::Unknown.into()),
            }
            // Parse finished; reset.
            reset_parsers_state(states);
            Ok(())
        }
        // Parse ended before the chunk did; reset.
        Ok(_) => {
            reset_parsers_state(states);
            Err(io::StatusWords::Unknown.into())
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
            comm.append(&[
                env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            ]);
            comm.append(b"alamgu example");
        }
        Ins::GetPubkey => {
            run_parser_apdu::<_, Bip32Key>(parser, get_get_address_state, &GET_ADDRESS_IMPL, comm)?
        }
        Ins::Sign => {
            run_parser_apdu::<_, SignParameters>(parser, get_sign_state, &SIGN_IMPL, comm)?
        }
        Ins::TestParsers => run_parser_apdu::<_, TestParsersSchema>(
            parser,
            get_test_parsers_state,
            &test_parsers_parser(),
            comm,
        )?,
        Ins::GetVersionStr => {
            comm.append(concat!("Alamgu Example ", env!("CARGO_PKG_VERSION")).as_ref());
        }
        Ins::Exit => nanos_sdk::exit_app(0),
    }
    Ok(())
}
