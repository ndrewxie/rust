// skip-filecheck
// EMIT_MIR_FOR_EACH_PANIC_STRATEGY
// unit-test: CopyProp

#![feature(custom_mir, core_intrinsics)]
#![allow(unused_assignments)]
extern crate core;
use core::intrinsics::mir::*;

struct NotCopy(bool);

// EMIT_MIR custom_move_arg.f.CopyProp.diff
#[custom_mir(dialect = "analysis", phase = "post-cleanup")]
fn f(_1: NotCopy) {
    mir!({
        let _2 = _1;
        Call(RET = opaque(Move(_1)), ReturnTo(bb1), UnwindContinue())
    }
    bb1 = {
        let _3 = Move(_2);
        Call(RET = opaque(_3), ReturnTo(bb2), UnwindContinue())
    }
    bb2 = {
        Return()
    })
}

#[inline(never)]
fn opaque<T>(_t: T) {}

fn main() {
    f(NotCopy(true));
    println!("hi");
}
