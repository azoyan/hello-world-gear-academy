#![no_std]
use gstd::{debug, msg, prelude::*, ActorId};
use hello_world_io::{Tamagotchi, TmgAction, TmgEvent};

static mut CONTRACT: Option<Tamagotchi> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let name = String::from_utf8(msg::load_bytes()
       .expect("Can't load init message"))
       .expect("Invalid message");


    CONTRACT = Some(Tamagotchi {
        name,
        date_of_birth: gstd::exec::block_timestamp(),
    });
}

#[no_mangle]
unsafe extern "C" fn handle() {
    let message: TmgAction = msg::load().expect("Wrong input to handle");
    let contract = common_state();

    let reply = match message {
        TmgAction::Name => TmgEvent::Name(contract.name.clone()),
        TmgAction::Age => TmgEvent::Age(contract.date_of_birth),
    };

    msg::reply(reply, 0).expect("Can't send TmgEvent reply");
}

pub fn common_state() -> &'static mut Tamagotchi {
    unsafe { CONTRACT.get_or_insert(Default::default()) }
}

#[no_mangle]
extern "C" fn state() {
    let greeting = unsafe { CONTRACT.get_or_insert(Default::default()) };
    msg::reply(greeting, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}
