// Copyright Jacob Salzberg
// SPDX-License-Identifier: Apache-2.0

// Basic test from the stacked borrows paper
#![allow(non_snake_case)]

fn example1(x: &mut i32, y: &mut i32) -> i32 {
    let x_rename = &mut *x;
    let y_rename = &mut *y;
    *x_rename = 42;
    *y_rename = 13;
    *x
}

fn main() {
    let mut local = 5;
    let raw_pointer = &mut local as *mut i32;
    let raw_pointer__pointer = &local as *const i32;
    let x__pointer = &local as *const i32;

    let result = unsafe {
        example1(&mut *raw_pointer, &mut *raw_pointer, &mut sstate)
    };
    assert_eq!(result, 13);

}
