// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// pretty-expanded FIXME #23616

#![allow(unknown_features)]
#![feature(box_syntax)]

use std::{option, mem};

// Iota-reduction is a rule in the Calculus of (Co-)Inductive Constructions,
// which "says that a destructor applied to an object built from a constructor
// behaves as expected".  -- http://coq.inria.fr/doc/Reference-Manual006.html
//
// It's a little more complicated here, because of pointers and regions and
// trying to get assert failure messages that at least identify which case
// failed.

enum E<T> { Thing(isize, T), Nothing((), ((), ()), [i8; 0]) }
impl<T> E<T> {
    fn is_none(&self) -> bool {
        match *self {
            E::Thing(..) => false,
            E::Nothing(..) => true
        }
    }
    fn get_ref(&self) -> (isize, &T) {
        match *self {
            E::Nothing(..) => panic!("E::get_ref(Nothing::<{}>)",  stringify!(T)),
            E::Thing(x, ref y) => (x, y)
        }
    }
}

macro_rules! check_option {
    ($e:expr, $T:ty) => {{
        check_option!($e, $T, |ptr| assert!(*ptr == $e));
    }};
    ($e:expr, $T:ty, |$v:ident| $chk:expr) => {{
        assert!(option::Option::None::<$T>.is_none());
        let e = $e;
        let s_ = option::Option::Some::<$T>(e);
        let $v = s_.as_ref().unwrap();
        $chk
    }}
}

macro_rules! check_fancy {
    ($e:expr, $T:ty) => {{
        check_fancy!($e, $T, |ptr| assert!(*ptr == $e));
    }};
    ($e:expr, $T:ty, |$v:ident| $chk:expr) => {{
        assert!(E::Nothing::<$T>((), ((), ()), [23; 0]).is_none());
        let e = $e;
        let t_ = E::Thing::<$T>(23, e);
        match t_.get_ref() {
            (23, $v) => { $chk }
            _ => panic!("Thing::<{}>(23, {}).get_ref() != (23, _)",
                       stringify!($T), stringify!($e))
        }
    }}
}

macro_rules! check_type {
    ($($a:tt)*) => {{
        check_option!($($a)*);
        check_fancy!($($a)*);
    }}
}

pub fn main() {
    check_type!(&17, &isize);
    check_type!(box 18, Box<isize>);
    check_type!("foo".to_string(), String);
    check_type!(vec!(20, 22), Vec<isize> );
    let mint: usize = unsafe { mem::transmute(main) };
    check_type!(main, fn(), |pthing| {
        assert!(mint == unsafe { mem::transmute(*pthing) })
    });
}
