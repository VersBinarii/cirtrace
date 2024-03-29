use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn get_args<'a>() -> ArgMatches<'a> {
    let host = Arg::with_name("host")
        .required(true)
        .help("SBC host to connect.")
        .takes_value(true);

    let search_term = Arg::with_name("search-term")
        .required(false)
        .multiple(true)
        .long("search-term")
        .short("S")
        .help("Filter trace based on this term. Can be phone number or IP.")
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
        .required_unless("status")
        .help("The name of the module instance.")
        .takes_value(true);

    let instance = Arg::with_name("instance")
        .required(false)
        .short("i")
        .long("instance")
        .requires("module")
        .help("Process instance")
        .takes_value(true);

    let output_file = Arg::with_name("output-file")
        .required(false)
        .short("o")
        .long("output-file")
        .help("Path location to store the output.")
        .takes_value(true);

    let sip_command = SubCommand::with_name("sip")
        .help("Prints raw captured SIP packets.")
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::UnifiedHelpMessage,
        ])
        .arg(search_term.clone());

    let trace_command = SubCommand::with_name("trace")
        .help("Prints full trace.")
        .arg(search_term);

    let status = SubCommand::with_name("status").help("Show remote host info.");

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
        .arg(output_file)
        .subcommand(sip_command)
        .subcommand(trace_command)
        .subcommand(status)
        .setting(AppSettings::SubcommandsNegateReqs)
        .get_matches()
}
