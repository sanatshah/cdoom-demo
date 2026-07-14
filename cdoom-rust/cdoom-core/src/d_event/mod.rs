//! Event queue ported from `d_event.c`.

use std::cell::UnsafeCell;
use std::ptr;

const MAX_EVENTS: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Event {
    pub event_type: i32,
    pub data1: i32,
    pub data2: i32,
    pub data3: i32,
    pub data4: i32,
    pub data5: i32,
    pub data6: i32,
}

struct EventState {
    events: [Event; MAX_EVENTS],
    head: usize,
    tail: usize,
}

struct EventQueue(UnsafeCell<EventState>);

unsafe impl Sync for EventQueue {}

static EVENT_QUEUE: EventQueue = EventQueue(UnsafeCell::new(EventState {
    events: [Event {
        event_type: 0,
        data1: 0,
        data2: 0,
        data3: 0,
        data4: 0,
        data5: 0,
        data6: 0,
    }; MAX_EVENTS],
    head: 0,
    tail: 0,
}));

pub fn post_event(event: Event) {
    // SAFETY: Chocolate Doom's event queue is process-global and used from the
    // same single-threaded input/game loop as the original C static storage.
    let state = unsafe { &mut *EVENT_QUEUE.0.get() };
    state.events[state.head] = event;
    state.head = (state.head + 1) % MAX_EVENTS;
}

pub fn pop_event() -> *mut Event {
    // SAFETY: See `post_event`; returned pointers are into stable static storage,
    // matching the C API's pointer-to-ring-slot contract.
    let state = unsafe { &mut *EVENT_QUEUE.0.get() };

    if state.tail == state.head {
        return ptr::null_mut();
    }

    let result = &mut state.events[state.tail] as *mut Event;
    state.tail = (state.tail + 1) % MAX_EVENTS;
    result
}

pub fn reset_for_tests() {
    // SAFETY: Tests reset the same process-global queue between cases.
    let state = unsafe { &mut *EVENT_QUEUE.0.get() };
    state.events = [Event::default(); MAX_EVENTS];
    state.head = 0;
    state.tail = 0;
}
