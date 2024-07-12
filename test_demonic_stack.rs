// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// TODO: Check that pointer object is not lost through pointer casts.

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]
#![feature(const_trait_impl)]
#![cfg_attr(not(kani), feature(register_tool))]
#![cfg_attr(not(kani), register_tool(kani))]
use std::ptr::null;

const STACK_DEPTH: usize = 15;
type PointerId = u32;
type StackItemKind = u32;

const KIND_UNIQUE: StackItemKind = 0;
const KIND_SHARED_RW: StackItemKind = 1;

type PointerValueKind = u32;
const KIND_IDENTIFIED : StackItemKind = 0;
const KIND_NONE: StackItemKind = 1;

#[cfg(any(kani))]
fn demonic_nondet() -> bool {
    kani::any::<bool>()
}

#[cfg(not(kani))]
fn demonic_nondet() -> bool {
    true
}


#[cfg(any(kani))]
fn same_pointer<T, U>(ptr1: *const T, ptr2: *const U) -> bool {
    kani::mem::pointer_object(ptr1) == kani::mem::pointer_object(ptr2)
}

#[cfg(not(kani))]
fn same_pointer<T, U>(ptr1: *const T, ptr2: *const U) -> bool {
    (ptr1 as *const _ as *const u8) == (ptr2 as *const _ as *const u8)
}

static mut SSTATE_MONITOR_OBJECT: *const u8 = null();
static mut SSTATE_MONITOR_OFFSET: usize = 0;
static mut SSTATE_MONITOR_ON: bool = false;
static mut SSTATE_STACK_IDS: [PointerId; STACK_DEPTH] = [0; STACK_DEPTH];
static mut SSTATE_STACK_KINDS: [StackItemKind; STACK_DEPTH] = [0; STACK_DEPTH];
static mut SSTATE_STACK_TOPS: usize = 0;
static mut SSTATE_NEXT_PTR_ID: PointerId = 0;

pub fn push_shared<U>(ptr: *const U, offset: usize, size: usize) {
    assert!(offset < size);
    unsafe {
        // switch monitor to this one
        {
            if demonic_nondet() {
                SSTATE_MONITOR_OBJECT = ptr as *const _ as *const u8;
                SSTATE_MONITOR_ON = true;
                let mut i = offset;
                let mut target = offset;
                while i < size { if demonic_nondet() { target = i }; i += 1; }
                SSTATE_MONITOR_OFFSET = target;
            }
        }
        {
            if same_pointer(SSTATE_MONITOR_OBJECT, ptr) && offset <= SSTATE_MONITOR_OFFSET && SSTATE_MONITOR_OFFSET < size && SSTATE_MONITOR_ON {
                    let top = SSTATE_STACK_TOPS;
                    assert!(top < STACK_DEPTH);
                    SSTATE_STACK_TOPS += 1;
            }
        }
    }
}

pub fn push_unique<U>(ptr: *const U, size: usize) -> PointerId {
    unsafe {
        {
            if demonic_nondet() {
                SSTATE_MONITOR_OBJECT = ptr as *const _ as *const u8;
                SSTATE_MONITOR_ON = true;
                let mut i = 0;
                let mut offset = 0;
                while i < size { if demonic_nondet() { offset = i }; i += 1; }
                SSTATE_MONITOR_OFFSET = offset;
            }
        }
        {
            let ptr_id_old = SSTATE_NEXT_PTR_ID;
            if same_pointer(SSTATE_MONITOR_OBJECT, ptr) &&
               SSTATE_MONITOR_OFFSET < size && SSTATE_MONITOR_ON  {
                        let top = SSTATE_STACK_TOPS;
                        assert!(top < STACK_DEPTH);
                        SSTATE_STACK_KINDS[top] = KIND_UNIQUE;
                        SSTATE_STACK_IDS[top] = ptr_id_old;
                        SSTATE_STACK_TOPS += 1;
                        SSTATE_NEXT_PTR_ID += 1;
            }
            ptr_id_old
        }
    }
}

fn use_2<U>(ptr: *const U, offset: usize, size: usize, kind: PointerValueKind, id: PointerId) {
    unsafe {
        if same_pointer(SSTATE_MONITOR_OBJECT, ptr) && offset < SSTATE_MONITOR_OFFSET && SSTATE_MONITOR_OFFSET < size &&
            SSTATE_MONITOR_ON {
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
            assert!(found, "Stack violated.");
        }
    }
}

fn new_mutable_ref<U>(loc: *const U, size: usize, kind: PointerValueKind, tag: PointerId) -> PointerId {
    use_2(loc, 0, size, kind, tag);
    push_unique(loc, size)
}

fn new_mutable_raw<U>(loc: *const U, offset: usize, size: usize, kind: PointerValueKind, tag: PointerId) -> PointerId {
    use_2(loc, offset, size, kind, tag);
    push_shared(loc, offset, size);
    0
}

static mut PARAM_1__POINTER: *const u8 = null();
static mut PARAM_1__POINTER_OFFSET: usize = 0;
static mut PARAM_1__POINTER_SIZE: usize = 0;
static mut PARAM_1__POINTER_KIND: PointerValueKind = 0;
static mut PARAM_1__TAG: PointerId = 0;
static mut PARAM_2__POINTER: *const u8 = null();
static mut PARAM_2__POINTER_OFFSET: usize = 0;
static mut PARAM_2__POINTER_SIZE: usize = 0;
static mut PARAM_2__POINTER_KIND: PointerValueKind = 0;
static mut PARAM_2__TAG: PointerId = 0;

fn get_param_1__pointer() -> *const u8 {
    unsafe { PARAM_1__POINTER }
}

fn get_param_1__pointer_offset() -> usize {
    unsafe { PARAM_1__POINTER_OFFSET }
}

fn get_param_1__pointer_size() -> usize {
    unsafe { PARAM_1__POINTER_SIZE }
}

fn set_param_1__pointer<T>(v: *const T) {
    unsafe { PARAM_1__POINTER = v as *const _ as *const u8 };
}

fn set_param_1__pointer_offset(v: usize) {
    unsafe { PARAM_1__POINTER_OFFSET = v };
}

fn set_param_1__pointer_size(v: usize) {
    unsafe { PARAM_1__POINTER_SIZE = v };
}

fn get_param_1__pointer_kind() -> PointerValueKind {
    unsafe { PARAM_1__POINTER_KIND }
}

fn set_param_1__pointer_kind(v: PointerValueKind) {
    unsafe { PARAM_1__POINTER_KIND = v };
}

fn get_param_1__tag() -> PointerId {
    unsafe { PARAM_1__TAG }
}

fn set_param_1__tag(v: PointerId) {
    unsafe { PARAM_1__TAG = v }
}

fn get_param_2__pointer() -> *const u8 {
    unsafe { PARAM_2__POINTER }
}

fn get_param_2__pointer_offset() -> usize {
    unsafe { PARAM_2__POINTER_OFFSET }
}

fn get_param_2__pointer_size() -> usize {
    unsafe { PARAM_2__POINTER_SIZE }
}

fn set_param_2__pointer<T>(v: *const T) {
    unsafe { PARAM_2__POINTER = v as *const _ as *const u8 };
}

fn set_param_2__pointer_offset(v: usize) {
    unsafe { PARAM_2__POINTER_OFFSET = v };
}

fn set_param_2__pointer_size(v: usize) {
    unsafe { PARAM_2__POINTER_SIZE = v };
}

fn get_param_2__pointer_kind() -> PointerValueKind {
    unsafe { PARAM_2__POINTER_KIND }
}

fn set_param_2__pointer_kind(v: PointerValueKind) {
    unsafe { PARAM_2__POINTER_KIND = v }
}

fn get_param_2__tag() -> PointerId {
    unsafe { PARAM_2__TAG }
}

fn set_param_2__tag(v: PointerId) {
    unsafe { PARAM_2__TAG = v };
}

fn example1(x: &mut i32, y: &mut i32) -> i32 {
    let x__pointer = get_param_1__pointer();
    let x__pointer_offset = get_param_1__pointer_offset();
    let x__pointer_size = get_param_1__pointer_size();
    let x__pointer_kind = get_param_1__pointer_kind();
    let x__tag = get_param_1__tag();
    let y__pointer = get_param_2__pointer();
    let y__pointer_offset= get_param_2__pointer_offset();
    let y__pointer_size = get_param_2__pointer_size();
    let y__pointer_kind = get_param_2__pointer_kind();
    let y__tag = get_param_2__tag();

    let x_rename = &mut *x;
    let x_rename__pointer = x__pointer.clone();
    let x_rename__pointer_offset = x__pointer_offset;
    let x_rename__pointer_size = x__pointer_size;
    let x_rename__pointer_kind = KIND_IDENTIFIED;
    let x_rename__pointer_id = new_mutable_ref(x__pointer, x__pointer_size, x__pointer_kind, x__tag);

    let y_rename = &mut *y;
    let y_rename__pointer = y__pointer.clone();
    let y_rename__pointer_offset = y__pointer_offset;
    let y_rename__pointer_size = y__pointer_size;
    let y_rename__pointer_kind = KIND_IDENTIFIED;
    let y_rename__pointer_id = new_mutable_ref(y__pointer, x__pointer_size, y__pointer_kind, y__tag);
    *x_rename = 42;
    use_2(x_rename__pointer, x_rename__pointer_offset, x_rename__pointer_size, x_rename__pointer_kind, x_rename__pointer_id);
    *y_rename = 13;
    use_2(y_rename__pointer, y_rename__pointer_offset, y_rename__pointer_size, y_rename__pointer_kind, y_rename__pointer_id);
    *x
}

#[kani::proof]
fn main() {
    let mut local = 5;
    let local__size = std::mem::size_of_val(&local);
    let _local__offset = 0;
    let local__pointer = &local as *const i32;
    // Steps involved in creating pointer:
    // create a ref to a local,
    // create a raw pointer to the ref,
    // create the pointer to this.
    let local__pointer_kind = KIND_IDENTIFIED;
    let local__tag = push_unique(local__pointer, local__size);

    let raw_pointer = &mut local as *mut i32;
    let _temporary_ref__size = std::mem::size_of_val(&local);
    let _temporary_ref__offset = 0;
    let temporary_ref__pointer = &local as *const i32;
    let temporary_ref__pointer_kind = KIND_IDENTIFIED;
    let temporary_ref__tag = new_mutable_ref(local__pointer, local__size, local__pointer_kind, local__tag);

    let raw_pointer__pointer = &local as *const i32;
    let raw_pointer__size = std::mem::size_of_val(&local);
    let raw_pointer__offset = 0;
    let raw_pointer__pointer_kind = KIND_NONE;
    let raw_pointer__tag = new_mutable_raw(temporary_ref__pointer, raw_pointer__offset, raw_pointer__size, temporary_ref__pointer_kind, temporary_ref__tag);

    set_param_1__pointer(&local);
    set_param_1__pointer_size(raw_pointer__size);
    set_param_1__pointer_offset(raw_pointer__offset);
    set_param_1__pointer_kind(KIND_IDENTIFIED);
    set_param_1__tag(new_mutable_ref(raw_pointer__pointer, raw_pointer__size, raw_pointer__pointer_kind, raw_pointer__tag));

    set_param_2__pointer(&local);
    set_param_2__pointer_size(raw_pointer__size);
    set_param_2__pointer_offset(raw_pointer__offset);
    set_param_2__pointer_kind(KIND_IDENTIFIED);
    set_param_2__tag(new_mutable_ref(raw_pointer__pointer, raw_pointer__size, raw_pointer__pointer_kind, raw_pointer__tag));

    let result = unsafe {
        example1(&mut *raw_pointer,
                 &mut *raw_pointer)
    };
    assert_eq!(result, 13);
}
