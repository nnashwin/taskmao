# taskmao
> Gain power through noticing.  Notice how you spend your time.

## Install
```
cargo install taskmao
```

## Usage 
```
$ taskmao --help
    USAGE:                                                                      taskmao [OPTIONS] [DESC] [SUBCOMMAND]

    ARGS:
        <DESC>    Sets the description of a task to execute

    FLAGS:
        -h, --help       Print help information
        -V, --version    Print version information

    OPTIONS:
        -p, --project <PROJECT>
        -t, --time <START_TIME>

    SUBCOMMANDS:
        delete    deletes a task that has the specified id
        end       ends currently executing task
        find      finds a previously executed task by id
        help      Print this message or the help of the given subcommand(s)
        info      returns info on the currently executing task or nothing 
        list      lists tasks completed / worked on today
```

## Maintainers
- [Tyler Boright](https://tylerboright.com)