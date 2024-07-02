// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]
use std::fmt;

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


fn pointer_object<U: Sized>(ptr: *const U) -> usize {
    0
}

fn pointer_offset<U: Sized>(ptr: *const U) -> usize {
    0
}

#[derive(Debug)]
struct SState {
    stack_ids: [[[PtrId; STACK_DEPTH]; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
    stack_kinds: [[[StackItemKind; STACK_DEPTH]; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
    stack_tops: [[usize; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
    next_ptr_id: PtrId,
}

impl fmt::Display for SState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ids: {:?}\n kinds: {:?}\n next ptr id: {}",
               &self.stack_ids[0][0][0..self.stack_tops[0][0]],
               &self.stack_kinds[0][0][0..self.stack_tops[0][0]],
               self.next_ptr_id)
    }
}

impl Default for SState {
    fn default() -> SState {
        SState {
            stack_ids: [[[0; STACK_DEPTH]; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
            stack_kinds: [[[KIND_SHARED_RW; STACK_DEPTH]; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
            stack_tops: [[0; MAX_OBJECT_SIZE]; MAX_NUM_OBJECTS],
            next_ptr_id: 0,
        }
    }
}

impl SState {
    pub fn push_shared<U>(&mut self, ptr: *const U) {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        assert!(obj < MAX_NUM_OBJECTS);
        assert!(offset < MAX_OBJECT_SIZE);
        let top = self.stack_tops[obj][offset];
        assert!(top < STACK_DEPTH);
        self.stack_tops[obj][offset] += 1;
    }

    pub fn push_unique<U>(&mut self, ptr: *const U) -> PtrId {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        assert!(obj < MAX_NUM_OBJECTS);
        assert!(offset < MAX_OBJECT_SIZE);
        let top = self.stack_tops[obj][offset];
        assert!(top < STACK_DEPTH);
        self.stack_kinds[obj][offset][top] = KIND_UNIQUE;
        let ptr_id_old = self.next_ptr_id;
        self.stack_ids[obj][offset][top] = ptr_id_old;
        self.stack_tops[obj][offset] += 1;
        self.next_ptr_id += 1;
        ptr_id_old
    }

    fn use_2<U>(&mut self, ptr: *const U, kind: PointerValueKind, id: PtrId) {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        assert!(obj < MAX_NUM_OBJECTS);
        assert!(offset < MAX_OBJECT_SIZE);
        let top = self.stack_tops[obj][offset];
        let mut found = false;
        if kind == KIND_IDENTIFIED {
            let mut i = 0;
            let mut new_top = 0;
            while (i < STACK_DEPTH) && (i < top) {
                if self.stack_kinds[obj][offset][i] == KIND_UNIQUE && self.stack_ids[obj][offset][i] == id {
                    new_top = i+1;
                    found = true;
                }
                i += 1;
            }
            self.stack_tops[obj][offset] = new_top;
        } else {
            let mut i = 0;
            let mut new_top = 0;
            while (i < STACK_DEPTH) && (i < top) {
                if self.stack_kinds[obj][offset][i] == KIND_SHARED_RW {
                    new_top = i+1;
                    found = true;
                }
                i += 1;
            }
            self.stack_tops[obj][offset] = new_top;
        }
        assert!(found);
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
    println!("Stack State, line 132: {}", sstate);
    let x_rename = &mut *x;
    let x_rename__pointer = x__pointer;
    let x_rename__pointer_kind = KIND_IDENTIFIED;
    let x_rename__pointer_id = sstate.new_mutable_ref(x__pointer, x__pointer_kind, x__id);

    println!("Stack State, line 138: {}", sstate);
    let y_rename = &mut *y;
    let y_rename__pointer = y__pointer;
    let y_rename__pointer_kind = KIND_IDENTIFIED;
    let y_rename__pointer_id = sstate.new_mutable_ref(y__pointer, y__pointer_kind, y__id);

    println!("Stack State, line 144: {}", sstate);
    *x_rename = 42;

    println!("Stack State, line 147: {}", sstate);
    sstate.use_2(x_rename__pointer, x_rename__pointer_kind, x_rename__pointer_id);
    *y_rename = 13;

    println!("Stack State, line 151: {}", sstate);
    sstate.use_2(y_rename__pointer, y_rename__pointer_kind, y_rename__pointer_id);
    *x
}

fn main() {
    let mut sstate = SState::default();
    println!("Stack State, line 151: {}", sstate);
    let mut local = 5;
    let local__pointer = &local as *const i32;
    let local__pointer_kind = KIND_IDENTIFIED;
    let local__id = sstate.push_unique(local__pointer);

    println!("Stack State, line 157: {}", sstate);
    let raw_pointer = &mut local as *mut i32;
    let temporary_ref__pointer = &local as *const i32;
    let temporary_ref__pointer_kind = KIND_IDENTIFIED;
    let temporary_ref__id = sstate.new_mutable_ref(local__pointer, local__pointer_kind, local__id);

    println!("Stack State, line 163: {}", sstate);
    let raw_pointer__pointer = &local as *const i32;
    let raw_pointer__pointer_kind = KIND_NONE;
    let raw_pointer__id = sstate.new_mutable_raw(temporary_ref__pointer, temporary_ref__pointer_kind, temporary_ref__id);

    println!("Stack State, line 168: {}", sstate);
    let x__pointer = &local as *const i32;
    let x__pointer_kind = KIND_IDENTIFIED;
    let x__id = sstate.new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    println!("Stack State, line 173: {}", sstate);
    let y__pointer = &local as *const i32;
    let y__pointer_kind = KIND_IDENTIFIED;
    let y__id = sstate.new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    println!("Stack State, line 178: {}", sstate);
    let result = unsafe {
        example1(&mut *raw_pointer, x__pointer, x__pointer_kind, x__id,
                 &mut *raw_pointer, y__pointer, y__pointer_kind, y__id,
                 &mut sstate)
    };
    assert_eq!(result, 13);

}
