use std::env;
use std::io::{self, BufRead};

use anyhow::{anyhow, Context, Result};

use simple_cron::{get_next_time, Specification, Specifier, MINUTES_IN_HOUR};

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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let raw_time = args.get(1).expect("Expected one argument to be given.");
    let raw_parts: Vec<_> = raw_time.splitn(2, ':').collect();
    let hours: usize = raw_parts
        .get(0)
        .expect("Expected hours in raw string.")
        .parse()
        .expect("Expected hours to be a number.");
    let minutes: usize = raw_parts
        .get(1)
        .expect("Expected minutes in raw string.")
        .parse()
        .expect("Expected minutes to be a number.");
    let time = hours * MINUTES_IN_HOUR + minutes;

    let stdin = io::stdin();
    for (index, line) in stdin.lock().lines().enumerate() {
        let line = line.with_context(|| format!("Failed to get line {}", index))?;
        let (minute, hour, target) =
            parse_line(&line).with_context(|| format!("Failed to parse input line {}", index))?;
        let specification = Specification::new(minute, hour);
        let (next_time, day) = get_next_time(specification, time);
        println!(
            "{}:{} {} - {}",
            next_time / MINUTES_IN_HOUR,
            next_time % MINUTES_IN_HOUR,
            day,
            target
        );
    }

    Ok(())
}
