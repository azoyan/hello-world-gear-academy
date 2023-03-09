#![no_std]

use codec::{Decode, Encode};
use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId, ReservationId};
use scale_info::TypeInfo;

pub type AttributeId = u32;
pub type Price = u128;
pub type TamagotchiId = ActorId;
pub type TransactionId = u64;

pub struct ProgramMetadata;
impl Metadata for ProgramMetadata {
    type Init = InOut<String, ()>;
    type Reply = InOut<(), ()>;
    type Others = InOut<(), ()>;
    type Signal = ();
    type Handle = InOut<TmgAction, TmgEvent>;
    type State = Tamagotchi;
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmgAction {
    Name,
    Age,
    Feed,
    Play,
    Sleep,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    ApproveTokens {
        account: ActorId,
        amount: u128,
    },
    SetFTokenContract(ActorId),
    BuyAttribute {
        store_id: ActorId,
        attribute_id: AttributeId,
    },
    CheckState,
    ReserveGas {
        reservation_amount: u64,
        duration: u32,
    },
    Owner,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum TmgEvent {
    Name(String),
    Age(u64),
    Fed,
    Entertained,
    Slept,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    ApproveTokens { account: ActorId, amount: u128 },
    ApprovalError,
    SetFTokenContract,
    AttributeBought(AttributeId),
    CompletePrevPurchase(AttributeId),
    ErrorDuringPurchase,
    FeedMe,
    PlayWithMe,
    WantToSleep,
    MakeReservation,
    GasReserved,
    Owner { owner: ActorId },
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub rested: u64,
    pub rested_block: u64,
    pub allowed_account: Option<ActorId>,
    pub ft_contract_id: ActorId,
    pub ft_transaction_id: TransactionId,
    pub approve_transaction: Option<(TransactionId, ActorId, u128)>,
    pub reservations: Vec<ReservationId>,
}

// pub struct AttributeStore {
//     admin: ActorId,
//     ft_contract_id: ActorId,
//     attributes: BTreeMap<AttributeId, (Metadata, Price)>,
//     owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
// }

// impl Tamagotchi {
//    pub async fn buy_attribute(&mut self, attribute_id: AttributeId) {
//         let (transaction_id, attribute_id) = if let Some((transaction_id, prev_attribute_id)) =
//             self.transactions.get(&msg::source())
//         {
//             // if `prev_attribute_id` is not equal to `attribute_id` then it means that transaction didn`t completed
//             // we ask the tamagotchi contract to complete the previous transaction
//             if attribute_id != *prev_attribute_id {
//                 msg::reply(
//                     StoreEvent::CompletePrevTx {
//                         attribute_id: *prev_attribute_id,
//                     },
//                     0,
//                 )
//                 .expect("Error in sending a reply `StoreEvent::CompletePrevTx`");
//                 return;
//             }
//             (*transaction_id, *prev_attribute_id)
//         } else {
//             let current_transaction_id = self.transaction_id;
//             self.transaction_id = self.transaction_id.wrapping_add(1);
//             self.transactions
//                 .insert(msg::source(), (current_transaction_id, attribute_id));
//             (current_transaction_id, attribute_id)
//         };

//         let result = self.sell_attribute(transaction_id, attribute_id).await;
//         self.transactions.remove(&msg::source());

//         msg::reply(StoreEvent::AttributeSold { success: result }, 0)
//             .expect("Error in sending a reply `StoreEvent::AttributeSold`");
//     }
// }
