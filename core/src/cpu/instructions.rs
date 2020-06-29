use super::{CPU, IO};

pub(super) type InstructionHandler<T> = fn(&mut CPU, &mut IO, T);

// TODO: Replace with const generics and trait specialization
pub trait InstructionFlag {
    fn bool() -> bool;
    fn num() -> u32;
}

macro_rules! compose_instr_handler {
    ($handler: ident, $skeleton: expr, $($bit: expr),* ) => {
        compose_instr_handler!($handler, flags => (), values => ( $($skeleton >> $bit & 0x1 != 0),* ))
    };
    ($handler: ident, flags => ( $( $flag: ident),* ), values => ()) => {
        CPU::$handler::<$($flag,)*>
    };
    ($handler: ident, flags => ( $($flag: ident),* ), values => ( $cur_value:expr $( , $value: expr )* )) => {
        if $cur_value {
            compose_instr_handler!($handler, flags => ( $($flag,)* InstrFlagSet ), values => ( $($value),* ))
        } else {
            compose_instr_handler!($handler, flags => ( $($flag,)* InstrFlagClear ), values => ( $($value),* ))
        }
    };
}

pub(super) struct InstrFlagSet {}
impl InstructionFlag for InstrFlagSet { fn bool() -> bool { true } fn num() -> u32 { 1 }}
pub(super) struct InstrFlagClear {}
impl InstructionFlag for InstrFlagClear { fn bool() -> bool { false } fn num() -> u32 { 0 } }
