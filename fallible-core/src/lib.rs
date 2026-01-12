#![no_std]

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FailurePointId(pub u32);

#[cold]
#[inline(never)]
pub fn simulated_failure<E>(id: FailurePointId) -> E {
    // todo: replace with proper structured reporting later
    panic!("fallible simulated failure {:?}", id.0);
}