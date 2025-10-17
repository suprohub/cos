use crate::{BinOp, Const, Key, UnOp};

pub const FRACTION_COUNT: u8 = 2;
// Default pos need to be on number 5
// Coords is in format (x, y)
pub const DEFAULT_POS: (u8, u8) = (2, 3);

#[rustfmt::skip]
pub fn keyboard_layout() -> [[Key; 6]; 6] {
    [
        [Const::Tau.into(), Const::EGamma.into(), Const::Pi.into(), Const::E.into(), Const::Phi.into(), Const::Sqrt2.into()],
        [UnOp::Sqrt.into(), Key::Num(7),          Key::Num(8),      Key::Num(9),     BinOp::Div.into(), Key::None],
        [UnOp::Neg.into(),  Key::Num(4),          Key::Num(5),      Key::Num(6),     BinOp::Mul.into(), Key::None],
        [UnOp::Pow2.into(), Key::Num(1),          Key::Num(2),      Key::Num(3),     BinOp::Add.into(), Key::None],
        [Key::None,         Key::Dot,             Key::Num(0),      Key::Result,     BinOp::Sub.into(), Key::None],
        [Key::None,         Key::None,            Key::None,        Key::None,       Key::None,         Key::None],
    ]
}
