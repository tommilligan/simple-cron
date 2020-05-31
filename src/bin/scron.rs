use std::env;
use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use chrono::{NaiveTime, Timelike};

use simple_cron::{get_next_time, Specification, Specifier};

/// Parse a single specification line of the form `* 0 target`
fn parse_line(line: &str) -> Result<(Specifier, Specifier, &str)> {
    let raw_parts: Vec<_> = line.splitn(3, ' ').collect();
    let minute = Specifier::from_str_max(
        raw_parts
            .get(0)
            .with_context(|| format!("No minute value."))?,
        60,
    )
    .with_context(|| format!("Invalid minute specifier."))?;
    let hour = Specifier::from_str_max(
        raw_parts
            .get(1)
            .with_context(|| format!("No hour value."))?,
        24,
    )
    .with_context(|| format!("Invalid hour specifier."))?;
    let target: &str = *raw_parts
        .get(2)
        .with_context(|| format!("No target value."))?;

    Ok((minute, hour, target))
}

/// For each line from the reader, calculate the correct output and send it to
/// writer.
fn run<Reader: BufRead, Writer: Write>(
    reader: Reader,
    writer: &mut Writer,
    current_time: &NaiveTime,
) -> Result<()> {
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to get line {}", index))?;
        let (minute, hour, target) =
            parse_line(&line).with_context(|| format!("Failed to parse input line {}", index))?;
        let specification = Specification::new(minute, hour);
        let (next_time, day) = get_next_time(&specification, current_time);
        // TODO(tommilligan) The hours are not padded here specifically
        // to make the given example in the task pass.
        writer.write(
            format!(
                "{}:{:02} {} - {}\n",
                next_time.hour(),
                next_time.minute(),
                day,
                target
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}

/// Deal with I/O, thin wrapper around `run`.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let raw_time = args.get(1).expect("Expected one argument to be given.");
    let current_time = NaiveTime::parse_from_str(raw_time, "%H:%M")?;

    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    run(reader, &mut writer, &current_time)
}

#[cfg(test)]
mod tests {
    use proptest::{
        prop_oneof, proptest,
        strategy::{BoxedStrategy, Just, Strategy},
    };

    use super::*;

    fn line_strategy() -> BoxedStrategy<String> {
        (
            prop_oneof![Just("*".to_owned()), (0..60u32).prop_map(|n| n.to_string())],
            prop_oneof![Just("*".to_owned()), (0..24u32).prop_map(|n| n.to_string())],
            "\\PC+",
        )
            .prop_map(|(minute, hour, target)| format!("{} {} {}\n", minute, hour, target))
            .boxed()
    }

    // Let's fuzz the input lines to check our parsing logic is sound
    // For this test we don't care if the output is correct, just that
    // it doesn't crash.
    proptest! {
        #[test]
        fn test_input_fuzz(
            line in line_strategy(),
        ) {
            // TODO(tommilligan) Quick hack to dump some proptest examples for benching.
            // See if there's a better way to do this?
            let mut writer = Vec::new();
            run(
                line.as_bytes(),
                &mut writer,
                &NaiveTime::from_hms(12, 34, 0),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_task_example() {
        let mut writer = Vec::new();
        run(
            r#"30 1 /bin/run_me_daily
45 * /bin/run_me_hourly
* * /bin/run_me_every_minute
* 19 /bin/run_me_sixty_times
"#
            .as_bytes(),
            &mut writer,
            &NaiveTime::from_hms(16, 10, 0),
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            r#"1:30 tomorrow - /bin/run_me_daily
16:45 today - /bin/run_me_hourly
16:10 today - /bin/run_me_every_minute
19:00 today - /bin/run_me_sixty_times
"#
        );
    }
}
