// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]

const STACK_DEPTH: usize = 15;
const MAX_NUM_OBJECTS: usize = 1024;
const MAX_OBJECT_SIZE: usize = 64;
type PtrId = u32;
type StackItemKind = u32;

const KIND_UNIQUE: StackItemKind = 0;
const KIND_SHARED_RW: StackItemKind = 1;

type PointerValueKind = u32;
const KIND_IDENTIFIED : StackItemKind = 0;
const KIND_NONE: StackItemKind = 1;


/*
// Uncomment to debug with println
use std::collections::HashMap;
static mut MY_MAP : Option<HashMap<usize, usize>> = None;
static mut MY_COUNT : usize = 0;
*/

fn pointer_object<U: Sized>(ptr: *const U) -> usize {
    kani::mem::pointer_object(ptr)
    /*
    // Uncomment to debug with println
    unsafe {
        match MY_MAP {
            Some(_) => {},
            None => MY_MAP = Some(HashMap::new()),
        };
        match &mut MY_MAP {
            Some(ref mut h) => {
                let mut found = false;
                let mut output = MY_COUNT;
                let ptr_v = std::mem::transmute::<*const U, usize>(ptr);
                match (h.get(&ptr_v)) {
                    Some(v) => { output = *v; found = true },
                    None => { }
                }
                if !found {
                    h.insert(ptr_v, MY_COUNT);
                    MY_COUNT += 1
                }
                output
            }
            None => { unreachable!() },
        }
    }
    // ptr as *const _ as usize
    */
}

fn pointer_offset<U: Sized>(ptr: *const U) -> usize {
    kani::mem::pointer_offset(ptr)
}

#[derive(Debug)]
struct SState {
    monitor_object: usize,
    monitor_offset: usize,
    monitor_on: bool,
    stack_ids: [PtrId; STACK_DEPTH],
    stack_kinds: [StackItemKind; STACK_DEPTH],
    stack_tops: usize,
    next_ptr_id: PtrId,
}

impl Default for SState {
    fn default() -> SState {
        SState {
            monitor_object: 0,
            monitor_offset: 0,
            monitor_on: false,
            stack_ids: [0; STACK_DEPTH],
            stack_kinds: [0; STACK_DEPTH],
            stack_tops: 0,
            next_ptr_id: 0
        }
    }
}

impl SState {
    pub fn push_shared<U>(&mut self, ptr: *const U) {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        // switch monitor to this one
        if kani::any::<bool>() {
            self.monitor_object = obj;
            self.monitor_offset = offset;
            self.monitor_on = true;
        }
        if self.monitor_object == obj && self.monitor_offset == offset && self.monitor_on {
            let top = self.stack_tops;
            assert!(top < STACK_DEPTH);
            self.stack_tops += 1;
        }
    }

    pub fn push_unique<U>(&mut self, ptr: *const U) -> PtrId {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        if kani::any::<bool>() {
            self.monitor_object = obj;
            self.monitor_offset = offset;
            self.monitor_on = true;
        }

        if self.monitor_object == obj && self.monitor_offset == offset && self.monitor_on {
            let top = self.stack_tops;
            assert!(top < STACK_DEPTH);
            self.stack_kinds[top] = KIND_UNIQUE;
            let ptr_id_old = self.next_ptr_id;
            self.stack_ids[top] = ptr_id_old;
            self.stack_tops += 1;
            self.next_ptr_id += 1;
            ptr_id_old
        } else {
            0
        }
    }

    fn use_2<U>(&mut self, ptr: *const U, kind: PointerValueKind, id: PtrId) {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);

        if self.monitor_object == obj && self.monitor_offset == offset && self.monitor_on {
            let top = self.stack_tops;
            let mut found = false;
            if kind == KIND_IDENTIFIED {
                let mut i = 0;
                let mut new_top = 0;
                while (i < STACK_DEPTH) && (i < top) {
                    if self.stack_kinds[i] == KIND_UNIQUE && self.stack_ids[i] == id {
                        new_top = i+1;
                        found = true;
                    }
                    i += 1;
                }
                self.stack_tops = new_top;
            } else {
                let mut i = 0;
                let mut new_top = 0;
                while (i < STACK_DEPTH) && (i < top) {
                    if self.stack_kinds[i] == KIND_SHARED_RW {
                        new_top = i+1;
                        found = true;
                    }
                    i += 1;
                }
                self.stack_tops = new_top;
            }
            assert!(found);
        }
    }

    fn new_mutable_ref<U>(&mut self, loc: *const U, kind: PointerValueKind, tag: PtrId) -> PtrId {
        self.use_2(loc, kind, tag);
        self.push_unique(loc)
    }

    fn new_mutable_raw<U>(&mut self, loc: *const U, kind: PointerValueKind, tag: PtrId) -> PtrId {
        self.use_2(loc, kind, tag);
        self.push_shared(loc);
        0
    }
}

fn example1(x: &mut i32, x__pointer: *const i32, x__pointer_kind: PointerValueKind, x__id: PtrId,
            y: &mut i32, y__pointer: *const i32, y__pointer_kind: PointerValueKind, y__id: PtrId,
            sstate: &mut SState) -> i32 {
    let x_rename = &mut *x;
    let x_rename__pointer = x__pointer;
    let x_rename__pointer_kind = KIND_IDENTIFIED;
    let x_rename__pointer_id = sstate.new_mutable_ref(x__pointer, x__pointer_kind, x__id);

    let y_rename = &mut *y;
    let y_rename__pointer = y__pointer;
    let y_rename__pointer_kind = KIND_IDENTIFIED;
    let y_rename__pointer_id = sstate.new_mutable_ref(y__pointer, y__pointer_kind, y__id);
    *x_rename = 42;
    sstate.use_2(x_rename__pointer, x_rename__pointer_kind, x_rename__pointer_id);
    *y_rename = 13;
    sstate.use_2(y_rename__pointer, y_rename__pointer_kind, y_rename__pointer_id);
    *x
}

#[kani::proof]
fn main() {
    let mut sstate = SState::default();
    let mut local = 5;
    let local__pointer = &local as *const i32;
    let local__pointer_kind = KIND_IDENTIFIED;
    let local__id = sstate.push_unique(local__pointer);

    let raw_pointer = &mut local as *mut i32;
    let temporary_ref__pointer = &local as *const i32;
    let temporary_ref__pointer_kind = KIND_IDENTIFIED;
    let temporary_ref__id = sstate.new_mutable_ref(local__pointer, local__pointer_kind, local__id);

    let raw_pointer__pointer = &local as *const i32;
    let raw_pointer__pointer_kind = KIND_NONE;
    let raw_pointer__id = sstate.new_mutable_raw(temporary_ref__pointer, temporary_ref__pointer_kind, temporary_ref__id);

    let x__pointer = &local as *const i32;
    let x__pointer_kind = KIND_IDENTIFIED;
    let x__id = sstate.new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    let y__pointer = &local as *const i32;
    let y__pointer_kind = KIND_IDENTIFIED;
    let y__id = sstate.new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    let result = unsafe {
        example1(&mut *raw_pointer, x__pointer, x__pointer_kind, x__id,
                 &mut *raw_pointer, y__pointer, y__pointer_kind, y__id,
                 &mut sstate)
    };
    assert_eq!(result, 13);

}
