# simple-cron

## Challenge

We have a set of tasks, each running at least daily, which are scheduled with a simplified cron. We want to find when each of them will next run.

The scheduler config looks like this:

```log
30 1 /bin/run_me_daily
45 * /bin/run_me_hourly
* * /bin/run_me_every_minute
* 19 /bin/run_me_sixty_times
```

The first field is the minutes past the hour, the second field is the hour of the day and the third is the command to run. For both cases `*` means that it should run for all values of that field. In the above example run_me_daily has been set to run at 1:30am every day and run_me_hourly at 45 minutes past the hour every hour. The fields are whitespace separated and each entry is on a separate line.

The challange is to write a command line program that when fed this config to stdin and the simulated 'current time' in the format HH:MM as command line argument outputs the soonest time at which each of the commands will fire and whether it is today or tomorrow. When the task should fire at the simulated 'current time' then that is the time you should output, not the next one.

For example given the above examples as input and the simulated 'current time' command-line argument 16:10 the output should be:

```log
1:30 tomorrow - /bin/run_me_daily
16:45 today - /bin/run_me_hourly
16:10 today - /bin/run_me_every_minute
19:00 today - /bin/run_me_sixty_times
```

## Solution design

The source code is split into the following:

- `src/bin/scron.rs` is concerned with IO, parsing cli arguments and formatting the output.
    - due to Rust's nice IO traits, the `run` function can be reused for input fuzzing tests using in-memory buffers
- `src/lib.rs` is concerned with the actual logic of `cron`

There are two kinds of tests present:

- unit tests written while developing the solution, which manually cover the expected cases
- property-based tests using `proptest`, which aim to generate new or exciting cases automatically

## Benchmarking

256 random input line cases were generated using [proptest](https://github.com/AltSysrq/proptest).
These were replicated to form a 1,024,000 line input file.

The release binary was benchmarked with [hyperfine](https://github.com/sharkdp/hyperfine), with the following results:

```log
$ ./bench/bench.sh
...
Benchmark #1: cat bench/bench-input-large.lines | ./target/release/scron 09:00
  Time (mean ± σ):      1.595 s ±  0.029 s    [User: 1.290 s, System: 0.380 s]
  Range (min … max):    1.546 s …  1.650 s    10 runs
```

This gives an approximate speed of **642k lines per second**.
