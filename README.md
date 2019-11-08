# Cirpack SBC tracing helper

## Cirpack tracing

Helper CLI tool to simplify runing traces on the Cirpack platform. You can run this tool on your local machine.

## Description
It logs to the specified host using SSH and enables the debug for the specified process. Currently only ibcf and bgcf is supported.

You can specify a search term after the subcommand to display trace only for things that match the term:

``` shell
cirtrace 192.168.1.100 -T 20 -p omni -M ibcf_border sip -S <phone_number>
```

The diference between `-M` and `-m` is that `-M` refers to the process instance while `-m` to the process name: i.e. ibcf or bgcf. You need to pass in `-i` with `-m`.


## Installation

``` shell
cargo install cirtrace
```

## Usage

``` shell

cir_trace 0.1
versbinarii <versbinarii@gmail.com>
Cirpack call troubleshooting helper

USAGE:
    cirtrace [OPTIONS] <host> --module-name <module-name> --password <password> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --instance <instance>          Process instance
    -m, --module <module>              The name of the module process. [possible values: ibcf, bgcf]
    -M, --module-name <module-name>    The name of the module instance.
    -o, --output-file <output-file>    Path location to store the output.
    -p, --password <password>          User password
    -T, --trace-time <trace-time>      How long the debug should run for in seconds. Default: 15s
    -u, --username <username>          Username to log in as. Default: omni

ARGS:
    <host>    SBC host to connect.

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    sip
    trace
```
