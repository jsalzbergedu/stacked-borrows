// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]
#![feature(const_trait_impl)]
use std::ptr::null;
use std::ptr::addr_of_mut;
use std::ffi::c_void;

const STACK_DEPTH: usize = 15;
type PtrId = u32;
type StackItemKind = u32;

const KIND_UNIQUE: StackItemKind = 0;
const KIND_SHARED_RW: StackItemKind = 1;

type PointerValueKind = u32;
const KIND_IDENTIFIED : StackItemKind = 0;
const KIND_NONE: StackItemKind = 1;


#[allow(unused)]
fn pointer_object<U: Sized>(ptr: *const U) -> usize {
    kani::mem::pointer_object(ptr)
}

#[allow(unused)]
fn pointer_offset<U: Sized>(ptr: *const U) -> usize {
    kani::mem::pointer_offset(ptr)
}

fn demonic_nondet() -> bool {
    kani::any::<bool>()
}

static mut SSTATE_MONITOR_OBJECT: usize = 0;
static mut SSTATE_MONITOR_OFFSET: usize = 0;
static mut SSTATE_MONITOR_ON: bool = false;
static mut SSTATE_STACK_IDS: [PtrId; STACK_DEPTH] = [0; STACK_DEPTH];
static mut SSTATE_STACK_KINDS: [StackItemKind; STACK_DEPTH] = [0; STACK_DEPTH];
static mut SSTATE_STACK_TOPS: usize = 0;
static mut SSTATE_NEXT_PTR_ID: PtrId = 0;

pub fn push_shared<U>(ptr: *const U) {
    unsafe {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        // switch monitor to this one
        if demonic_nondet() {
            SSTATE_MONITOR_OBJECT = obj;
            SSTATE_MONITOR_OFFSET = offset;
            SSTATE_MONITOR_OBJECT = obj;
            SSTATE_MONITOR_ON = true;
        }
        if SSTATE_MONITOR_OBJECT == obj && SSTATE_MONITOR_OFFSET == offset && SSTATE_MONITOR_ON {
            let top = SSTATE_STACK_TOPS;
            assert!(top < STACK_DEPTH);
            SSTATE_STACK_TOPS += 1;
        }
    }
}

pub fn push_unique<U>(ptr: *const U) -> PtrId {
    unsafe {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);
        if demonic_nondet() {
            SSTATE_MONITOR_OBJECT = obj;
            SSTATE_MONITOR_OFFSET = offset;
            SSTATE_MONITOR_ON = true;
        }
        if  SSTATE_MONITOR_OBJECT == obj && SSTATE_MONITOR_OFFSET == offset && SSTATE_MONITOR_ON  {
            let top = SSTATE_STACK_TOPS;
            assert!(top < STACK_DEPTH);
            SSTATE_STACK_KINDS[top] = KIND_UNIQUE;
            let ptr_id_old = SSTATE_NEXT_PTR_ID;
            SSTATE_STACK_IDS[top] = ptr_id_old;
            SSTATE_STACK_TOPS += 1;
            SSTATE_NEXT_PTR_ID += 1;
            ptr_id_old
        } else {
            0
        }
    }
}

fn use_2<U>(ptr: *const U, kind: PointerValueKind, id: PtrId) {
    unsafe {
        let obj = pointer_object(ptr);
        let offset = pointer_offset(ptr);

        if SSTATE_MONITOR_OBJECT == obj && SSTATE_MONITOR_OFFSET == offset && SSTATE_MONITOR_ON {
            let top = SSTATE_STACK_TOPS;
            let mut found = false;
            if kind == KIND_IDENTIFIED {
                let mut i = 0;
                let mut new_top = 0;
                while (i < STACK_DEPTH) && (i < top) {
                    if SSTATE_STACK_KINDS[i] == KIND_UNIQUE && SSTATE_STACK_IDS[i] == id {
                        new_top = i+1;
                        found = true;
                    }
                    i += 1;
                }
                SSTATE_STACK_TOPS = new_top;
            } else {
                let mut i = 0;
                let mut new_top = 0;
                while (i < STACK_DEPTH) && (i < top) {
                    if SSTATE_STACK_KINDS[i] == KIND_SHARED_RW {
                        new_top = i+1;
                        found = true;
                    }
                    i += 1;
                }
                SSTATE_STACK_TOPS = new_top;
            }
            assert!(found);
        }
    }
}

fn new_mutable_ref<U>(loc: *const U, kind: PointerValueKind, tag: PtrId) -> PtrId {
    use_2(loc, kind, tag);
    push_unique(loc)
}

fn new_mutable_raw<U>(loc: *const U, kind: PointerValueKind, tag: PtrId) -> PtrId {
    use_2(loc, kind, tag);
    push_shared(loc);
    0
}

static mut X__POINTER: *const c_void = null();
static mut X__POINTER_KIND: PointerValueKind = 0;
static mut X__ID: PtrId = 0;
static mut Y__POINTER: *const c_void = null();
static mut Y__POINTER_KIND: PointerValueKind = 0;
static mut Y__ID: PtrId = 0;

fn example1(x: &mut i32, y: &mut i32) -> i32 {
    let x__pointer = unsafe { &mut *addr_of_mut!(X__POINTER) };
    let x__pointer_kind = unsafe { &mut *addr_of_mut!(X__POINTER_KIND) };
    let x__id = unsafe { &mut *addr_of_mut!(X__ID) };
    let y__pointer = unsafe { &mut *addr_of_mut!(Y__POINTER) };
    let y__pointer_kind = unsafe { &mut *addr_of_mut!(Y__POINTER_KIND) };
    let y__id = unsafe { &mut *addr_of_mut!(Y__ID) };

    let x_rename = &mut *x;
    let x_rename__pointer = x__pointer.clone();
    let x_rename__pointer_kind = KIND_IDENTIFIED;
    let x_rename__pointer_id = new_mutable_ref(*x__pointer, *x__pointer_kind, *x__id);

    let y_rename = &mut *y;
    let y_rename__pointer = y__pointer.clone();
    let y_rename__pointer_kind = KIND_IDENTIFIED;
    let y_rename__pointer_id = new_mutable_ref(*y__pointer, *y__pointer_kind, *y__id);
    *x_rename = 42;
    use_2(x_rename__pointer, x_rename__pointer_kind, x_rename__pointer_id);
    *y_rename = 13;
    use_2(y_rename__pointer, y_rename__pointer_kind, y_rename__pointer_id);
    *x
}

#[kani::proof]
fn main() {
    let mut local = 5;
    let local__pointer = &local as *const _ as *const c_void;
    let local__pointer_kind = KIND_IDENTIFIED;
    let local__id = push_unique(local__pointer);

    let raw_pointer = &mut local as *mut i32;
    let temporary_ref__pointer = &local as *const i32;
    let temporary_ref__pointer_kind = KIND_IDENTIFIED;
    let temporary_ref__id = new_mutable_ref(local__pointer, local__pointer_kind, local__id);

    let raw_pointer__pointer = &local as *const _ as *const c_void;
    let raw_pointer__pointer_kind = KIND_NONE;
    let raw_pointer__id = new_mutable_raw(temporary_ref__pointer, temporary_ref__pointer_kind, temporary_ref__id);

    let x__pointer = unsafe { &mut *addr_of_mut!(X__POINTER) };
    let x__pointer_kind = unsafe { &mut *addr_of_mut!(X__POINTER_KIND) };
    let x__id = unsafe { &mut *addr_of_mut!(X__ID) };
    *x__pointer = &local as *const _ as *const c_void;
    *x__pointer_kind = KIND_IDENTIFIED;
    *x__id = new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    let y__pointer = unsafe { &mut *addr_of_mut!(Y__POINTER) };
    let y__pointer_kind = unsafe { &mut *addr_of_mut!(Y__POINTER_KIND) };
    let y__id = unsafe { &mut *addr_of_mut!(Y__ID) };
    *y__pointer = &local as *const _ as *const c_void;
    *y__pointer_kind = KIND_IDENTIFIED;
    *y__id = new_mutable_ref(raw_pointer__pointer, raw_pointer__pointer_kind, raw_pointer__id);

    let result = unsafe {
        example1(&mut *raw_pointer,
                 &mut *raw_pointer)
    };
    assert_eq!(result, 13);
}
