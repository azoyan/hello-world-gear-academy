#![no_std]
use ft_logic_io::Action;
use ft_main_io::FTokenAction;
use gstd::{
    exec::{self, block_timestamp},
    msg,
    prelude::*,
    ReservationId,
};
use io::{Tamagotchi, TmgAction, TmgEvent};
use store_io::StoreAction;

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
        ft_contract_id: Default::default(),
        ft_transaction_id: 0,
        approve_transaction: None,
        reservations: vec![],
    });
}

#[gstd::async_main]
async fn main() {
    let message: TmgAction = msg::load().expect("Wrong input to handle");
    let contract = common_state();

    let current_timestamp = block_timestamp();
    msg::send_delayed(exec::program_id(), TmgAction::CheckState, 0, 60_u32)
        .expect("Error in sending a delayed message `TmgAction::CheckState`");
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
        TmgAction::ApproveTokens { account, amount } => {
            // 1. Approve tokens to the Store contract. Send message to FT contract

            let action = Action::Approve {
                approved_account: account,
                amount,
            }
            .encode();
            let payload = FTokenAction::Message {
                transaction_id: 1,
                payload: action,
            }
            .encode();
            let _reply = msg::send_for_reply(contract.ft_contract_id, payload, 0)
                .expect("Can't send message")
                .await
                .expect("Can't receive reply");

            TmgEvent::ApproveTokens { account, amount }
        }
        TmgAction::SetFTokenContract(ft_address) => {
            contract.ft_contract_id = ft_address;
            TmgEvent::SetFTokenContract
        }
        TmgAction::BuyAttribute {
            store_id,
            attribute_id,
        } => {
            let action = StoreAction::BuyAttribute { attribute_id };
            let payload = action.encode();
            // 3. Send message "Buy attribute" to Store contract
            let _reply = msg::send_for_reply(store_id, payload, 0)
                .expect("Can't send message")
                .await
                .expect("Can't receive reply");
            TmgEvent::AttributeBought(attribute_id)
        }
        TmgAction::CheckState => {
            if contract.fed < 100 {
                msg::reply(TmgEvent::FeedMe, 0).expect("Can't send TmgEvent reply");
                return;
            } else if contract.rested < 100 {
                msg::reply(TmgEvent::WantToSleep, 0).expect("Can't send TmgEvent reply");
                return;
            } else if contract.entertained < 100 {
                msg::reply(TmgEvent::PlayWithMe, 0).expect("Can't send TmgEvent reply");
                return;
            }
            panic!("What would we return?");
        }
        TmgAction::ReserveGas {
            reservation_amount,
            duration,
        } => {
            let reservation_id =
                ReservationId::reserve(reservation_amount, duration).expect("Can't reserve Gas");
            contract.reservations.push(reservation_id);
            TmgEvent::GasReserved
        }
        TmgAction::Owner => TmgEvent::Owner {
            owner: contract.owner,
        },
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

#[no_mangle]
extern "C" fn my_handle_signal() {
    let contract = unsafe { CONTRACT.get_or_insert(Default::default()) };

    let reservation_id = if !contract.reservations.is_empty() {
        contract.reservations.remove(0)
    } else {
        return;
    };

    msg::send_from_reservation(
        reservation_id,
        exec::program_id(),
        TmgEvent::MakeReservation,
        0,
    )
    .expect("Can't send TmgEvent::MakeReservation");
}
