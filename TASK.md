Thank you for taking on our simplified cron parser challenge. The aim of this exercise is to see how well you can take a spec and implement some working software. 

We have designed this question to be something that can be completed within two hours and we certainly do not expect you to give up more than two hours of your time. The most is that the core functionality works, feel free to leave notes on what you do if you had more time to complete the task.

We have a set of tasks, each running at least daily, which are scheduled with a simplified cron. We want to find when each of them will next run.

The scheduler config looks like this:

30 1 /bin/run_me_daily
45 * /bin/run_me_hourly
* * /bin/run_me_every_minute
* 19 /bin/run_me_sixty_times

The first field is the minutes past the hour, the second field is the hour of the day and the third is the command to run. For both cases * means that it should run for all values of that field. In the above example run_me_daily has been set to run at 1:30am every day and run_me_hourly at 45 minutes past the hour every hour. The fields are whitespace separated and each entry is on a separate line.

We want you to write a command line program that when fed this config to stdin and the simulated 'current time' in the format HH:MM as command line argument outputs the soonest time at which each of the commands will fire and whether it is today or tomorrow. When the task should fire at the simulated 'current time' then that is the time you should output, not the next one.

For example given the above examples as input and the simulated 'current time' command-line argument 16:10 the output should be

1:30 tomorrow - /bin/run_me_daily
16:45 today - /bin/run_me_hourly
16:10 today - /bin/run_me_every_minute
19:00 today - /bin/run_me_sixty_times

We will assess your solution as objectively as possible, to this end we run your solution through a test runner, and then take a look at the source for code clarity.

Feel free to use any language you want to complete the task. To run successfully against the test runner, we want to run something like this

cat input.txt | <how to run your solution> <simulated current time>
e.g. cat input.txt | python minicron.py 16:10

Please submit all code as attachments, not in the body of the email, as formatting is often lost or mangled. If you want to attach multiple files please do so as an archive (e.g. zip, tar, git bundle, etc).

Any questions please just get in touch!
