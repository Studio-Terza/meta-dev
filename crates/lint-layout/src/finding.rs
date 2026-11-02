use std::fmt;

use crate::fix::Fix;

pub trait Finding: fmt::Display {
    fn fix(&self) -> Fix;
}

impl<Found: Finding + ?Sized> Finding for &Found {
    fn fix(&self) -> Fix {
        (**self).fix()
    }
}
