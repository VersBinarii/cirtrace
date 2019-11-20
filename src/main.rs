#[macro_use]
extern crate lazy_static;

use error::{Error, TraceResult};
use std::{
    io::{self, Write},
    path::Path,
    thread,
    time::{Duration, Instant},
};

mod args;
mod commands;
mod error;
mod sip_parse;
mod ssh;

fn main() -> TraceResult<()> {
    let matches = args::get_args();

    let host = matches.value_of("host").unwrap();
    let username = matches.value_of("username").unwrap_or("omni");
    let password = matches.value_of("password").unwrap();
    let timeout: u32 = matches
        .value_of("trace-time")
        .unwrap_or("15")
        .parse()
        .unwrap();
    let process = matches.value_of("module");
    let process_name = matches.value_of("module-name");
    let instance = matches.value_of("instance");

    let cmd = commands::CommandRunner::new(host, username, password)?;

    let ps_out = match (process, process_name, instance) {
        (None, Some(pn), None) => cmd.get_ps_list(&[pn])?,
        (Some(p), None, Some(i)) => {
            cmd.get_ps_list(&[p, &format!("i{}", i)])?
        }
        (Some(p), None, None) => cmd.get_ps_list(&[p])?,
        _ => String::new(),
    };

    let pandi = find_process_and_instance(&ps_out);

    // Connect to node and set up the debugging
    match pandi {
        (Some(ref p), _, Some(ref i)) => {
            cmd.enable_debug(p, i).and_then(|_| {
                println!("Enabled debug mode");
                Ok(())
            })?;
        }
        _ => {}
    }

    // Get th time on remote system to the nearest minute
    let remote_time = cmd.get_remote_time()?;

    let pn = match pandi {
        (Some(ref p), None, _) => Some(p),
        (None, Some(ref pn), _) => Some(pn),
        (Some(_), Some(ref pn), _) => Some(pn),
        _ => None,
    };

    let trace_output = match pn {
        // Tail the trace file only from the moment we started the test
        Some(proc_name) => {
            wait(Duration::from_secs(timeout as u64));
            let trace_out = cmd.get_trace(&proc_name, &remote_time)?;
            cmd.disable_debug(&pandi.0.unwrap(), &pandi.2.unwrap())
                .and_then(|_| {
                    println!("Disabled debugging");
                    Ok(())
                })?;
            trace_out
        }
        None => "".to_string(),
    };

    match matches.subcommand() {
        ("sip", Some(s_match)) => {
            let search_terms: Vec<_> =
                if s_match.occurrences_of("search-term") > 0 {
                    s_match.values_of("search-term").unwrap().collect()
                } else {
                    vec![]
                };
            // We have a full trace now so we can now
            // extract interesting stuf from it
            let sip_parser = sip_parse::SipParser::new();
            let sip_packets =
                sip_parser.extract_sip(&trace_output, &search_terms, true);

            if matches.is_present("output-file") {
                let out_file = matches.value_of("output-file").unwrap();
                save_output_locally(&sip_packets, out_file)?;
            }

            for p in sip_packets.iter() {
                println!("{}", p);
            }
        }
        ("status", _) => {
            let _ = cmd.show_status();
        }
        _ => println!("{}", "Not supported yet"),
    };

    Ok(())
}

fn save_output_locally<T: std::fmt::Display, P: AsRef<Path> + Copy>(
    to_save: &[T],
    filepath: P,
) -> TraceResult<()> {
    use std::fs::OpenOptions;
    use std::io::BufWriter;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filepath)
        .map_err(|e| Error::File(e, filepath.as_ref().to_path_buf()))?;

    let mut writer = BufWriter::new(file);

    for p in to_save.iter() {
        let _ = writer
            .write(p.to_string().as_bytes())
            .map_err(Error::Write)?;
    }

    Ok(())
}

/// This should find ("process", "process-name", "instance number")
fn find_process_and_instance(
    s: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    // Extract the process from this:
    // omni     28848  0.0  8.6 770804 714432 ?       Sl    2018   0:50 /home/omni/bin/ibcf -i1 -ribcf_core -f/home/etc/ibcf_core.cfg -tpip=254

    let process = if let Some(idx1) = s.find(" /home") {
        if let Some(idx2) = s[idx1..].find(" -i") {
            if let Some(p) = s.get(idx1..idx1 + idx2) {
                p.split("/").into_iter().last().map(|s| s.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let process_name = if let Some(idx1) = s.find("-r") {
        if let Some(idx2) = s[idx1..].find(" ") {
            if let Some(p) = s.get(idx1..idx1 + idx2) {
                // Just skip the -r
                Some(p[2..].to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let instance = if let Some(idx1) = s.find("-i") {
        if let Some(idx2) = s[idx1..].find(" ") {
            if let Some(p) = s.get(idx1..idx1 + idx2) {
                // Just skip the -i
                Some(p[2..].to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    (process, process_name, instance)
}

fn wait(wait_time: Duration) {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    let start = Instant::now();

    println!("Awaiting test.\nElapsed: ");
    loop {
        if start.elapsed() > wait_time {
            println!("\n");
            break;
        }
        thread::sleep(Duration::from_secs(1));

        let _ = write!(handle, "{}{}", "\x1B[1000D", "\x1B[K");
        let _ = handle.flush();
        let _ = write!(handle, "{}s", start.elapsed().as_secs());
        let _ = handle.flush();
    }
}
