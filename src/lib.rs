use std::fmt;

use anyhow::{anyhow, Context, Result};
use chrono::{Duration, NaiveTime, Timelike};

/// Represents the day of the next trigger time.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Represents a single token in the cron specification.
#[derive(Clone, Debug)]
pub enum Specifier {
    Any,
    Only(u32),
}

impl Specifier {
    /// Convert '*' or an integer into a specifier, checking the integer is within
    /// the given range.
    pub fn from_str_max(raw_token: &str, max_ordinal: u32) -> Result<Specifier> {
        match raw_token {
            "*" => Ok(Specifier::Any),
            raw_token => {
                let number = raw_token
                    .parse()
                    .with_context(|| format!("Invalid number."))?;
                match number {
                    x if x < max_ordinal => Ok(Specifier::Only(number)),
                    _ => Err(anyhow!(
                        "Number {} outside of range {}.",
                        number,
                        max_ordinal
                    )),
                }
            }
        }
    }
}

/// Represents the complete time portion of the cron specification.
#[derive(Clone, Debug)]
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
    pub fn matches(&self, time: &NaiveTime) -> bool {
        if let Specifier::Only(spec) = self.minute {
            if spec != time.minute() {
                return false;
            }
        }
        if let Specifier::Only(spec) = self.hour {
            if spec != time.hour() {
                return false;
            }
        }
        true
    }
}

/// Given a cron specification and the current time, return the next
/// time this would be triggered.
// TODO(tommilligan) This could be optimised by returning Cow<'a, NaiveTime>
// for cases where the current time is valid
pub fn get_next_time(specification: &Specification, current_time: &NaiveTime) -> (NaiveTime, Day) {
    // Always check - if we match the current time, it's all good!
    if specification.matches(&current_time) {
        return (current_time.clone(), Day::Today);
    }

    // There are only 4 possible combinations, so let's just enumerate them!
    // If I was going to implement this with a larger spec, I'd do some sort of sweep-forward
    // strategy, starting from the largest unit of time and working down.
    let next_time = match &specification {
        // If the specifier is any, then we already returned above.
        Specification {
            minute: Specifier::Any,
            hour: Specifier::Any,
        } => panic!("all-Any specification didn't match current time."),
        // If we get a specific time, just construct it directly
        Specification {
            minute: Specifier::Only(minute),
            hour: Specifier::Only(hour),
        } => NaiveTime::from_hms(*hour, *minute, 0),
        // If we get any hour but a specific minute, the next trigger is either
        // this hour or the next hour
        Specification {
            minute: Specifier::Only(minute),
            hour: Specifier::Any,
        } => {
            let mut next_time = NaiveTime::from_hms(current_time.hour(), *minute, 0);
            // If the minute is behind the current minute, we need to add another hour
            if next_time.minute() < current_time.minute() {
                next_time = next_time + Duration::hours(1);
            }
            next_time
        }
        // If we get a specific hour but any minute, the trigger time must be
        // the start of that hour
        Specification {
            minute: Specifier::Any,
            hour: Specifier::Only(hour),
        } => NaiveTime::from_hms(*hour, 0, 0),
    };

    // If the next time is behind the current time, it must be tomorrow
    // as NaiveTime always wraps over the date boundary
    let day = if &next_time < current_time {
        Day::Tomorrow
    } else {
        Day::Today
    };
    (next_time, day)
}

#[cfg(test)]
mod tests {
    use proptest::{
        prop_assert, prop_oneof, proptest,
        strategy::{BoxedStrategy, Just, Strategy},
    };

    use super::*;

    fn specifier_strategy(max_ordinal: u32) -> BoxedStrategy<Specifier> {
        prop_oneof![
            Just(Specifier::Any),
            (0..max_ordinal).prop_map(|n| Specifier::Only(n)),
        ]
        .boxed()
    }

    fn specification_strategy() -> BoxedStrategy<Specification> {
        (specifier_strategy(60), specifier_strategy(24))
            .prop_map(|(minute, hour)| Specification { minute, hour })
            .boxed()
    }

    fn time_strategy() -> BoxedStrategy<NaiveTime> {
        (0..24u32, 0..60u32)
            .prop_map(|(hour, minute)| NaiveTime::from_hms(hour, minute, 0))
            .boxed()
    }

    // Lets pick a random spec and start time, and get the next time.
    // Then check the following invariants:
    // - The returned time actually matches the pattern
    // - There are no earlier matches
    proptest! {
        #[test]
        fn test_no_earlier_matches(
            specification in specification_strategy(),
            current_time in time_strategy()
        ) {
            let (next_time, day) = get_next_time(&specification, &current_time);
            // Check our return value actually matches
            prop_assert!(
                specification.matches(&next_time),
                "Next time {} doesn't match specification.",
                next_time,
            );
            // Check for earlier values
            let mut check_time = next_time.clone();
            let mut check_day = day.clone();
            loop {
                if (&check_time, &check_day) == (&current_time, &Day::Today) {
                    // we reached our starting time without incident
                    break;
                }

                // Move back one step
                if check_time == NaiveTime::from_hms(0, 0, 0) && check_day == Day::Tomorrow {
                    check_day = Day::Today;
                };
                check_time = check_time - Duration::minutes(1);

                // Check if we have a new match
                prop_assert!(
                    !specification.matches(&check_time),
                    "Said next time was {:?}, but found earlier match {:?}.",
                    (&next_time, &day),
                    (&check_time, &check_day)
                );
            }
        }
    }

    #[test]
    fn test_spec_any_minute_specific_hour() {
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Any, Specifier::Only(12)),
                &NaiveTime::from_hms(12, 00, 0),
            ),
            (NaiveTime::from_hms(12, 00, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Any, Specifier::Only(15)),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(15, 00, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Any, Specifier::Only(9)),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(09, 00, 0), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_specific_minute_any_hour() {
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(0), Specifier::Any),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(12, 00, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(7), Specifier::Any),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(12, 07, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(7), Specifier::Any),
                &NaiveTime::from_hms(23, 57, 0)
            ),
            (NaiveTime::from_hms(00, 07, 0), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_specific_minute_specific_hour() {
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(0), Specifier::Only(12)),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(12, 00, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(13), Specifier::Only(13)),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(13, 13, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Only(11), Specifier::Only(11)),
                &NaiveTime::from_hms(12, 00, 0)
            ),
            (NaiveTime::from_hms(11, 11, 0), Day::Tomorrow)
        );
    }

    #[test]
    fn test_spec_any_minute_any_hour() {
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Any, Specifier::Any),
                &NaiveTime::from_hms(00, 00, 0)
            ),
            (NaiveTime::from_hms(00, 00, 0), Day::Today)
        );
        assert_eq!(
            get_next_time(
                &Specification::new(Specifier::Any, Specifier::Any),
                &NaiveTime::from_hms(23, 59, 0)
            ),
            (NaiveTime::from_hms(23, 59, 0), Day::Today)
        );
    }
}
