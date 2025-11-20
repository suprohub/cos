use crate::{BinOp, Const, Key, UnOp};

pub const FRACTION_COUNT: u8 = 2;
// Default pos need to be on number 5
// Coords is in format (x, y)
pub const DEFAULT_POS: (u8, u8) = (2, 3);

#[rustfmt::skip]
pub fn keyboard_layout() -> [[Key; 7]; 7] {
    [
        [UnOp::Sin.into(), Key::None,              Const::Phi.into(),    Const::Tau.into(), Const::Sqrt2.into(), Key::None,         Key::None],
        [UnOp::Cos.into(), Key::None,              Const::EGamma.into(), Const::Pi.into(),  Const::E.into(),     Key::None,         Key::None],
        [UnOp::Tan.into(), UnOp::Sqrt.into(),      Key::Num(7),          Key::Num(8),       Key::Num(9),         BinOp::Div.into(), Key::None],
        [Key::None,        UnOp::Neg.into(),       Key::Num(4),          Key::Num(5),       Key::Num(6),         BinOp::Mul.into(), Key::None],
        [Key::None,        UnOp::Pow2.into(),      Key::Num(1),          Key::Num(2),       Key::Num(3),         BinOp::Add.into(), Key::None],
        [Key::None,        UnOp::Pow3.into(),      Key::Dot,             Key::Num(0),       Key::Result,         BinOp::Sub.into(), Key::None],
        [Key::None,        UnOp::Factorial.into(), Key::Clear,           Key::Delete,       Key::Reset,          Key::None,         Key::None],
    ]
}
