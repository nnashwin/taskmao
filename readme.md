# taskmao
> Gain power through noticing.  Notice how you spend your time.

## Install
```
cargo install taskmao
```

## Usage
```
$ taskmao --help
    Gain power through noticing.  Notice how you spend your time.

    Usage: taskmao [OPTIONS] [DESC] [COMMAND]

    Commands:
      delete  deletes a task by its unique id
      end     ends currently executing task
      find    finds a previously executed task by id
      info    returns info on the currently executing task or nothing
      list    lists tasks completed / worked on today
      help    Print this message or the help of the given subcommand(s)

    Arguments:
      [DESC]  sets the description of a task to execute;  only occurs if a subcommand is not matched from the list

    Options:
      -p, --project <project>  sets the project of a task [default: default]
      -t, --time <START_TIME>  manually set a start time for new task other than now
      -h, --help               Print help
```

## Maintainers
- [Norman Nashwin]
