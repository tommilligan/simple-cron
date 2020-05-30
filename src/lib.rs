use std::fmt;

use anyhow::{Context, Result};

pub const MINUTES_IN_HOUR: usize = 60;

#[derive(Debug, PartialEq, Eq)]
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

pub struct Specification {
    minute: Specifier,
    hour: Specifier,
}

impl Specification {
    pub fn new(minute: Specifier, hour: Specifier) -> Self {
        Self { minute, hour }
    }

    /// Return whether this specification matches the given time in
    /// minutes.
    pub fn matches(&self, time: usize) -> bool {
        if let Specifier::Only(spec) = self.minute {
            if spec != time % MINUTES_IN_HOUR {
                return false;
            }
        }
        if let Specifier::Only(spec) = self.hour {
            if spec != time / MINUTES_IN_HOUR {
                return false;
            }
        }
        true
    }
}

pub fn get_next_time(specification: Specification, current_time: usize) -> (usize, Day) {
    // Always check - if we match the current time, it's all good!
    if specification.matches(current_time) {
        return (current_time, Day::Today);
    }

    // There are only 4 possible combinations, so lets just enumerate them!
    match &specification {
        // If the specifier is any, then we already returned above.
        Specification {
            minute: Specifier::Any,
            hour: Specifier::Any,
        } => panic!("Any specification didn't match current time."),
        // If we get a specific time, just work out the next instance
        // (as this will be unique in a given day).
        Specification {
            minute: Specifier::Only(minute),
            hour: Specifier::Only(hour),
        } => {
            let next_time = hour * MINUTES_IN_HOUR + minute;
            let diff = next_time as isize - current_time as isize;
            let day = if diff < 0 { Day::Tomorrow } else { Day::Today };
            (next_time, day)
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_any() {
        assert_eq!(
            get_next_time(Specification::new(Specifier::Any, Specifier::Any), 0),
            (0, Day::Today)
        );
        assert_eq!(
            get_next_time(Specification::new(Specifier::Any, Specifier::Any), 42),
            (42, Day::Today)
        );
        assert_eq!(
            get_next_time(Specification::new(Specifier::Any, Specifier::Any), 1339),
            (1339, Day::Today)
        );
    }
}
