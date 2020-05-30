use std::fmt;

use anyhow::{Context, Result};

pub const MINUTES_IN_HOUR: usize = 60;

pub enum Day {
    Today,
    Tomorrow,
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Today => "today",
                Self::Tomorrow => "tomorrow",
            }
        )
    }
}

pub enum Specifier {
    Any,
    Only(usize),
}

pub fn get_next_time(minute: Specifier, hour: Specifier, current_time: usize) -> (usize, Day) {
    (0, Day::Today)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
