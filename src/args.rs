use clap::{App, Arg, ArgMatches, SubCommand};

pub fn get_args<'a>() -> ArgMatches<'a> {
    let host = Arg::with_name("host")
        .required(true)
        .help("SBC host to connect.")
        .takes_value(true);

    let ip = Arg::with_name("ip")
        .required(false)
        .help("Trace based on IP address")
        .takes_value(true);

    let username = Arg::with_name("username")
        .required(false)
        .short("u")
        .long("username")
        .help("Username to log in as. Default: omni ")
        .takes_value(true);

    let password = Arg::with_name("password")
        .required(true)
        .short("p")
        .long("password")
        .help("User password")
        .takes_value(true);

    let trace_time = Arg::with_name("trace-time")
        .required(false)
        .short("T")
        .long("trace-time")
        .help("How long the debug should run for in seconds. Default: 15s")
        .takes_value(true);

    let process = Arg::with_name("module")
        .required(false)
        .short("m")
        .long("module")
        .requires("instance")
        .possible_values(&["ibcf", "bgcf"])
        .help("The name of the module process.")
        .takes_value(true);

    let process_name = Arg::with_name("module-name")
        .required(false)
        .short("M")
        .long("module-name")
        .required_unless_all(&["module", "instance"])
        .help("The name of the module instance.")
        .takes_value(true);

    let instance = Arg::with_name("instance")
        .required(false)
        .short("i")
        .long("instance")
        .requires("module")
        .help("Process instance")
        .takes_value(true);

    let sip_command = SubCommand::with_name("sip")
        .help("Prints raw captured SIP packets.")
        .arg(ip.clone());

    let trace_command = SubCommand::with_name("trace")
        .help("Prints full trace.")
        .arg(ip);

    App::new("cir_trace")
        .version("0.1")
        .author("versbinarii <versbinarii@gmail.com>")
        .about("Cirpack call troubleshooting helper")
        .arg(host)
        .arg(username)
        .arg(password)
        .arg(trace_time)
        .arg(process)
        .arg(process_name)
        .arg(instance)
        .subcommand(sip_command)
        .subcommand(trace_command)
        .get_matches()
}
