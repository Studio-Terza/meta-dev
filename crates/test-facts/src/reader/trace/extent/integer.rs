use lint_layout::{Name, Word};

use crate::boundary::Quantity;
use crate::check::Rows;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Integer {
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    Unsigned128,
    Size,
    Signed8,
    Signed16,
    Signed32,
    Signed64,
    Signed128,
    SignedSize,
}

impl Integer {
    const ALL: [Integer; 12] = [
        Integer::Unsigned8,
        Integer::Unsigned16,
        Integer::Unsigned32,
        Integer::Unsigned64,
        Integer::Unsigned128,
        Integer::Size,
        Integer::Signed8,
        Integer::Signed16,
        Integer::Signed32,
        Integer::Signed64,
        Integer::Signed128,
        Integer::SignedSize,
    ];

    pub(crate) fn named(name: &Name) -> Option<Integer> {
        Self::ALL.iter().copied().find(|integer| *name == integer.word())
    }

    fn word(self) -> Word<'static> {
        match self {
            Integer::Unsigned8 => Word::new("u8"),
            Integer::Unsigned16 => Word::new("u16"),
            Integer::Unsigned32 => Word::new("u32"),
            Integer::Unsigned64 => Word::new("u64"),
            Integer::Unsigned128 => Word::new("u128"),
            Integer::Size => Word::new("usize"),
            Integer::Signed8 => Word::new("i8"),
            Integer::Signed16 => Word::new("i16"),
            Integer::Signed32 => Word::new("i32"),
            Integer::Signed64 => Word::new("i64"),
            Integer::Signed128 => Word::new("i128"),
            Integer::SignedSize => Word::new("isize"),
        }
    }

    pub(crate) fn rows(self) -> Rows {
        match self {
            Integer::Unsigned8 => Rows::over(Quantity::new(i128::from(u8::MIN)), Quantity::new(i128::from(u8::MAX))),
            Integer::Unsigned16 => Rows::over(Quantity::new(i128::from(u16::MIN)), Quantity::new(i128::from(u16::MAX))),
            Integer::Unsigned32 => Rows::over(Quantity::new(i128::from(u32::MIN)), Quantity::new(i128::from(u32::MAX))),
            Integer::Unsigned64 | Integer::Size => {
                Rows::over(Quantity::new(i128::from(u64::MIN)), Quantity::new(i128::from(u64::MAX)))
            }
            Integer::Unsigned128 => Rows::over(Quantity::ZERO, Quantity::new(i128::MAX)),
            Integer::Signed8 => Rows::over(Quantity::new(i128::from(i8::MIN)), Quantity::new(i128::from(i8::MAX))),
            Integer::Signed16 => Rows::over(Quantity::new(i128::from(i16::MIN)), Quantity::new(i128::from(i16::MAX))),
            Integer::Signed32 => Rows::over(Quantity::new(i128::from(i32::MIN)), Quantity::new(i128::from(i32::MAX))),
            Integer::Signed64 | Integer::SignedSize => {
                Rows::over(Quantity::new(i128::from(i64::MIN)), Quantity::new(i128::from(i64::MAX)))
            }
            Integer::Signed128 => Rows::over(Quantity::new(i128::MIN), Quantity::new(i128::MAX)),
        }
    }
}
