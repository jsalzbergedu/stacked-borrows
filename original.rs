// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]

#[kani::proof]
fn main() {
    let mut local = 5;
    let raw_pointer = &mut local as *mut i32;

    let result = unsafe {
        example1(&mut *raw_pointer, &mut *raw_pointer)
    };
    assert_eq!(result, 13);
}


fn example1(x: &mut i32, y: &mut i32) -> i32 {
    *x = 42;
    *y = 13;
    *x
}
