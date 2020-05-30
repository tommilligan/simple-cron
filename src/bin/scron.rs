use std::env;
use std::io::{self, BufRead, Write};

use anyhow::{anyhow, Context, Result};

use simple_cron::{get_next_time, Specification, Specifier, Time};

/// Convert '*' or an integer into a specifier, checking the integer is within
/// the given range.
fn parse_token(raw_token: &str, max_ordinal: usize) -> Result<Specifier> {
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

/// Parse a single specification line of the form `* 0 target`
fn parse_line(line: &str) -> Result<(Specifier, Specifier, &str)> {
    let raw_parts: Vec<_> = line.splitn(3, ' ').collect();
    let minute = parse_token(
        raw_parts
            .get(0)
            .with_context(|| format!("No minute value."))?,
        60,
    )
    .with_context(|| format!("Invalid minute specifier."))?;
    let hour = parse_token(
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
    current_time: &Time,
) -> Result<()> {
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to get line {}", index))?;
        let (minute, hour, target) =
            parse_line(&line).with_context(|| format!("Failed to parse input line {}", index))?;
        let specification = Specification::new(minute, hour);
        let (next_time, day) = get_next_time(&specification, current_time);
        writer.write(format!("{} {} - {}\n", next_time, day, target).as_bytes())?;
    }
    Ok(())
}

/// Deal with I/O, thin wrapper around `run`.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let raw_time = args.get(1).expect("Expected one argument to be given.");
    let current_time: Time = raw_time.parse()?;

    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    run(reader, &mut writer, &current_time)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            &"16:10".parse().unwrap(),
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
