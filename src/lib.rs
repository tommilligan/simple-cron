use std::fmt;
use std::str::FromStr;

use anyhow::{Context, Error, Result};

const MINUTES_IN_HOUR: usize = 60;
const MINUTES_IN_DAY: usize = 1440;

#[derive(PartialEq, Eq, Clone)]
pub struct Time {
    minute_in_day: usize,
}

impl Time {
    pub fn new(minute_in_day: usize) -> Self {
        Self {
            minute_in_day: minute_in_day % MINUTES_IN_DAY,
        }
    }

    pub fn minute_in_day(&self) -> usize {
        self.minute_in_day
    }

    pub fn hours(&self) -> usize {
        self.minute_in_day / MINUTES_IN_HOUR
    }

    pub fn minutes(&self) -> usize {
        self.minute_in_day % MINUTES_IN_HOUR
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hours(), self.minutes())
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Time {
    type Err = Error;

    fn from_str(raw_time: &str) -> Result<Self, Self::Err> {
        let raw_parts: Vec<_> = raw_time.splitn(2, ':').collect();
        let hours: usize = raw_parts
            .get(0)
            .with_context(|| format!("Expected hours in raw string."))?
            .parse()
            .with_context(|| format!("Expected hours to be a number."))?;
        let minutes: usize = raw_parts
            .get(1)
            .with_context(|| format!("Expected minutes in raw string."))?
            .parse()
            .with_context(|| format!("Expected minutes to be a number."))?;
        Ok(Self {
            minute_in_day: hours * MINUTES_IN_HOUR + minutes,
        })
    }
}

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
    pub fn matches(&self, time: &Time) -> bool {
        if let Specifier::Only(spec) = self.minute {
            if spec != time.minutes() {
                return false;
            }
        }
        if let Specifier::Only(spec) = self.hour {
            if spec != time.hours() {
                return false;
            }
        }
        true
    }
}

// TODO(tommilligan) This could be optimised by returning Cow<'a, Time>
// for cases where the current time is valid
pub fn get_next_time(specification: Specification, current_time: &Time) -> (Time, Day) {
    // Always check - if we match the current time, it's all good!
    if specification.matches(&current_time) {
        return (current_time.clone(), Day::Today);
    }

    // There are only 4 possible combinations, so let's just enumerate them!
    match &specification {
        // If the specifier is any, then we already returned above.
        Specification {
            minute: Specifier::Any,
            hour: Specifier::Any,
        } => panic!("all-Any specification didn't match current time."),
        // If we get a specific time, just work out the next instance
        // (as this will be unique in a given day).
        Specification {
            minute: Specifier::Only(minute),
            hour: Specifier::Only(hour),
        } => {
            let next_time = Time::new(hour * MINUTES_IN_HOUR + minute);
            let diff = next_time.minute_in_day() as isize - current_time.minute_in_day() as isize;
            let day = if diff < 0 { Day::Tomorrow } else { Day::Today };
            (next_time, day)
        }
        // If we get any hour but a specific minute, work out the next instance
        Specification {
            minute: Specifier::Only(minute),
            hour: Specifier::Any,
        } => {
            // If the minute is behind the current minute, we need to add another hour
            let next_time = (current_time.hours() + if minute < &current_time.minutes() {
                1
            } else {
                0
            }) * MINUTES_IN_HOUR
                + minute;
            let day = if next_time < MINUTES_IN_DAY {
                Day::Today
            } else {
                Day::Tomorrow
            };
            let next_time = Time::new(next_time);
            (next_time, day)
        }
        // If we get a specific hour but any minute, work out the start of the next hour
        Specification {
            minute: Specifier::Any,
            hour: Specifier::Only(hour),
        } => {
            // If the hour is behind the current hour, we need to add another day
            let next_time = Time::new(hour * MINUTES_IN_HOUR);
            let day = if hour < &current_time.hours() {
                Day::Tomorrow
            } else {
                Day::Today
            };
            (next_time, day)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_any_minute_specific_hour() {
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Any, Specifier::Only(12)),
                &"12:00".parse().unwrap()
            ),
            ("12:00".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Any, Specifier::Only(15)),
                &"12:00".parse().unwrap()
            ),
            ("15:00".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Any, Specifier::Only(9)),
                &"12:00".parse().unwrap()
            ),
            ("09:00".parse().unwrap(), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_specific_minute_any_hour() {
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(0), Specifier::Any),
                &"12:00".parse().unwrap()
            ),
            ("12:00".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(7), Specifier::Any),
                &"12:00".parse().unwrap()
            ),
            ("12:07".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(7), Specifier::Any),
                &"23:57".parse().unwrap()
            ),
            ("00:07".parse().unwrap(), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_specific() {
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(0), Specifier::Only(12)),
                &"12:00".parse().unwrap()
            ),
            ("12:00".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(13), Specifier::Only(13)),
                &"12:00".parse().unwrap()
            ),
            ("13:13".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Only(11), Specifier::Only(11)),
                &"12:00".parse().unwrap()
            ),
            ("11:11".parse().unwrap(), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_any() {
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Any, Specifier::Any),
                &"00:00".parse().unwrap()
            ),
            ("00:00".parse().unwrap(), Day::Today)
        );
        assert_eq!(
            get_next_time(
                Specification::new(Specifier::Any, Specifier::Any),
                &"23:59".parse().unwrap()
            ),
            ("23:59".parse().unwrap(), Day::Today)
        );
    }
}
