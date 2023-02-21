#![no_std]
use gstd::{exec::block_timestamp, msg, prelude::*};
use io::{Tamagotchi, TmgAction, TmgEvent};

const HUNGER_PER_BLOCK: u64 = 1; // how much Tamagochi becomes hungry for the block;
const ENERGY_PER_BLOCK: u64 = 2; // how much Tamagochi loses energy per block;
const BOREDOM_PER_BLOCK: u64 = 2; // how bored Tamagochigetsper block;

const FILL_PER_SLEEP: u64 = 1000; // how much energy Tamagochi gets per sleep;
const FILL_PER_FEED: u64 = 1000; // how much Tamagotchi becomes full during feeding;
const FILL_PER_ENTERTAINMENT: u64 = 1000; //  how much Tamagotchi becomes happy during feeding;

static mut CONTRACT: Option<Tamagotchi> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let name = String::from_utf8(msg::load_bytes().expect("Can't load init message"))
        .expect("Invalid message");

    CONTRACT = Some(Tamagotchi {
        name,
        date_of_birth: block_timestamp(),
        owner: msg::source(),
        fed: 10000,
        fed_block: block_timestamp(),
        entertained: 10000,
        entertained_block: block_timestamp(),
        rested: 10000,
        rested_block: BOREDOM_PER_BLOCK,
        allowed_account: None,
    });
}

#[no_mangle]
unsafe extern "C" fn handle() {
    let message: TmgAction = msg::load().expect("Wrong input to handle");
    let contract = common_state();

    let current_timestamp = block_timestamp();
    let reply = match message {
        TmgAction::Name => TmgEvent::Name(contract.name.clone()),
        TmgAction::Age => TmgEvent::Age(contract.date_of_birth),
        TmgAction::Feed => {
            contract.fed = contract.fed + FILL_PER_FEED
                - (current_timestamp - contract.fed_block) * HUNGER_PER_BLOCK;
            contract.fed_block = current_timestamp;
            TmgEvent::Fed
        }
        TmgAction::Play => {
            contract.entertained = contract.entertained + FILL_PER_ENTERTAINMENT
                - (current_timestamp - contract.entertained_block) * BOREDOM_PER_BLOCK;
            contract.entertained_block = current_timestamp;
            TmgEvent::Entertained
        }
        TmgAction::Sleep => {
            contract.rested = contract.rested + FILL_PER_SLEEP
                - (current_timestamp - contract.rested_block) * ENERGY_PER_BLOCK;
            contract.rested_block = current_timestamp;
            TmgEvent::Slept
        }
        TmgAction::Transfer(new_owner) => {
            contract.owner = new_owner;
            TmgEvent::Transfer(contract.owner)
        }
        TmgAction::Approve(allowed_account) => {
            contract.allowed_account = Some(allowed_account);
            TmgEvent::Approve(allowed_account)
        }
        TmgAction::RevokeApproval => {
            contract.allowed_account = None;
            TmgEvent::RevokeApproval
        }
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
