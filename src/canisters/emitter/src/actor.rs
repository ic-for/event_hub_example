
use candid::{CandidType, Deserialize};
use ic_cdk::storage::{stable_save, stable_restore};
use ic_cdk_macros::*;

use ic_event_hub::{*, event_hub::EventHub};
use ic_event_hub_macros::*;

// ------------------ STATE ------------------ 
#[derive(Default, CandidType, Deserialize)]
pub struct RequestCounter {
    pub counter: u64,
    pub latest: Vec<u8>,
}

static mut STATE: Option<RequestCounter> = None;

pub fn get_state() -> &'static mut RequestCounter {
    unsafe {
        STATE.as_mut().unwrap()
    }
}

#[init] 
fn init() {
    unsafe { STATE = Some(RequestCounter::default()); }
}

#[pre_upgrade]
fn pre_upgrade() {
    let canister_state = unsafe {
        STATE.take()
    };
    let event_hub_state = _take_event_hub_state();

    stable_save((canister_state, event_hub_state)).expect("Unable to stable save");
}

#[post_upgrade]
fn post_upgrade() {
    let (canister_state, event_hub_state): (Option<RequestCounter>, Option<EventHub>) = stable_restore().expect("Unable to stable load");

    unsafe {
        STATE = canister_state;
    }
    _put_event_hub_state(event_hub_state);
}

// ------------------- MAIN Logic -------------
#[update]
fn mirror(data: Vec<u8>) {
    get_state().counter += 1;
    get_state().latest = data.clone();

    emit(MirrorEvent { data });
}

#[query] 
fn get_requests_count() -> u64 {
    get_state().counter
}

#[query]
fn get_latest() -> String {
    String::from_utf8(get_state().latest.clone()).unwrap()
}

#[ic_cdk_macros::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// --------------------- EVENTS ------------------
#[derive(Event)]
pub struct MirrorEvent {
    pub data: Vec<u8>,
}
// maximum batch forming time = 20 seconds, maximum batch size = 500 KB
implement_event_emitter!(1_000_000_000 * 20, 500 * 1024);
implement_subscribe!();
implement_unsubscribe!();

#[heartbeat]
pub fn tick() {
    send_events();
}


