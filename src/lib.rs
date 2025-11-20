#![no_std]
// For logging
#![feature(sync_unsafe_cell)]

use ufmt::derive::uDebug;

use crate::num::Num;

pub mod config;
pub mod log;
pub mod num;

pub struct Calculator<const F: u8> {
    a: Num<F>,
    op: Option<Op>,
    b: Num<F>,
    frac: bool,
    frac_digits: u8,
}

impl<const F: u8> Default for Calculator<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const F: u8> Calculator<F> {
    pub const fn new() -> Self {
        Self {
            a: Num::ZERO,
            op: None,
            b: Num::ZERO,
            frac: false,
            frac_digits: 0,
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if `self.op` is none.
    pub fn handle_input(&mut self, key: Key) -> Result<Option<Num<F>>, CalcError> {
        match key {
            Key::Num(n) => {
                let v = if self.op.is_none() {
                    &mut self.a
                } else {
                    &mut self.b
                };

                if self.frac {
                    if self.frac_digits < F {
                        let scale_factor = 10i64.pow((F - self.frac_digits - 1) as u32);
                        *v += Num::from_raw((n as i64) * scale_factor);
                        self.frac_digits += 1;
                    }
                } else {
                    *v = *v * Num::from_int(10) + Num::from_int(n as i64);
                }
            }
            Key::Dot => {
                self.frac = true;
                self.frac_digits = 0;
            }
            Key::BinOp(op) => {
                self.op = Some(Op::BinOp(op));
                self.frac = false;
                self.frac_digits = 0;
            }
            Key::UnOp(op) => {
                self.op = Some(Op::UnOp(op));
                self.frac = false;
                self.frac_digits = 0;
                return Ok(Some(self.calc()?));
            }
            Key::Const(c) => {
                *if self.op.is_none() {
                    &mut self.a
                } else {
                    &mut self.b
                } = match c {
                    Const::Pi => Num::PI,
                    Const::Tau => Num::TAU,
                    Const::Phi => Num::PHI,
                    Const::EGamma => Num::EGAMMA,
                    Const::Sqrt2 => Num::SQRT_2,
                    Const::E => Num::E,
                }
            }
            Key::Result => {
                let result = self.calc()?;
                self.frac = false;
                self.frac_digits = 0;
                return Ok(Some(result));
            }
            Key::Delete => {
                let v = if self.op.is_none() {
                    &mut self.a
                } else {
                    &mut self.b
                };

                if self.frac {
                    if self.frac_digits > 0 {
                        let digit_scale = 10i64.pow((F - self.frac_digits) as u32);
                        let last_digit = (v.0 / digit_scale) % 10;
                        v.0 -= last_digit * digit_scale;
                        self.frac_digits -= 1;

                        if self.frac_digits == 0 {
                            self.frac = false;
                        }
                    } else {
                        self.frac = false;
                    }
                } else {
                    *v /= Num::from_int(10);
                }
            }
            Key::Clear => {
                *if self.op.is_none() {
                    &mut self.a
                } else {
                    &mut self.b
                } = Num::ZERO;

                self.frac = false;
                self.frac_digits = 0;
            }
            Key::Reset => {
                self.a = Num::ZERO;
                self.op = None;
                self.b = Num::ZERO;
                self.frac = false;
                self.frac_digits = 0;
            }
            _ => {}
        }

        Ok(None)
    }

    /// # Errors
    ///
    /// Will return `Err` if `self.op` is none.
    pub fn calc(&mut self) -> Result<Num<F>, CalcError> {
        let Some(op) = self.op.take() else {
            return Err(CalcError::Calc);
        };

        debug!("a = {}; op = {:?}; b = {}", self.a.0, self.op, self.b.0);

        let a = self.a;

        self.a = match op {
            Op::BinOp(op) => {
                let b = self.b;
                self.b = Num::ZERO;
                match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => a / b,
                }
            }
            Op::UnOp(op) => match op {
                UnOp::Neg => -a,
                UnOp::Sqrt => a.sqrt(),
                UnOp::Pow2 => a * a,
                UnOp::Factorial => a.factorial(),
            },
        };

        Ok(self.a)
    }
}

#[derive(uDebug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    None,
    Num(u8),
    Dot,
    BinOp(BinOp),
    UnOp(UnOp),
    Const(Const),
    Result,
    Delete,
    Clear,
    Reset,

    Photomath,
    GPT5,
}

impl From<BinOp> for Key {
    #[inline]
    fn from(v: BinOp) -> Self {
        Self::BinOp(v)
    }
}

impl From<UnOp> for Key {
    #[inline]
    fn from(v: UnOp) -> Self {
        Self::UnOp(v)
    }
}

impl From<Const> for Key {
    #[inline]
    fn from(v: Const) -> Self {
        Self::Const(v)
    }
}

#[derive(uDebug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    BinOp(BinOp),
    UnOp(UnOp),
}

#[derive(uDebug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(uDebug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Sqrt,
    Pow2,
    Factorial,
}

#[derive(uDebug, Clone, Copy, PartialEq, Eq)]
pub enum Const {
    Pi,
    Tau,
    Phi,
    EGamma,
    Sqrt2,
    E,
}

#[derive(Debug, uDebug, Clone, Copy, PartialEq, Eq)]
pub enum CalcError {
    Calc,
}
