use std::{
    io::{self, Write},
    net::{IpAddr, SocketAddr},
    thread,
    time::{Duration, Instant},
};

use error::TraceResult;

mod args;
mod error;
mod ssh;

fn main() -> TraceResult<()> {
    let matches = args::get_args();

    let host = matches.value_of("host").unwrap();
    let host = host.parse().expect("Failed to parse the IP address");
    let username = matches.value_of("username").unwrap_or("omni");
    let password = matches.value_of("password").unwrap();
    let timeout: u32 =
        matches.value_of("timeout").unwrap_or("15").parse().unwrap();
    let process = matches.value_of("module");
    let process_name = matches.value_of("module-name");
    let instance = matches.value_of("instance");

    let socket = SocketAddr::new(IpAddr::V4(host), 22);
    let mut ssh = ssh::SshClient::connect(socket, username, password)?;

    let ps_out = match (process, process_name, instance) {
        (None, Some(pn), None) => {
            ssh.send_cmd(&format!("ps aux | grep {}", pn))?
        }
        (Some(p), None, Some(i)) => {
            ssh.send_cmd(&format!("ps aux | grep {} | grep i{}", p, i))?
        }
        _ => unreachable!(),
    };

    let pandi = find_process_and_instance(&ps_out);
    println!("{:?}", pandi);
    // Connect to node and set up the debugging
    match pandi {
        (Some(ref p), _, Some(i)) => {
            let _ = ssh.send_cmd(&format!(
                "mgt_cscf -name={} -i{} -debug=3 -loglevel=0",
                p, i
            ))?;
            println!("Enabled debug mode");
        }
        _ => {
            println!("Could not find proces and instance to match on.");
            std::process::exit(1)
        }
    }

    wait(Duration::from_secs(timeout as u64));

    let pn = match pandi {
        (Some(ref p), None, _) => p,
        (None, Some(ref pn), _) => pn,
        (Some(_), Some(ref pn), _) => pn,
        _ => unreachable!(),
    };
    let trace_output = ssh.send_cmd(&format!("cat /home/log/{}.1", pn))?;

    println!("Capturing trace output");

    let _ = ssh.send_cmd(&format!(
        "mgt_cscf -name={} -i{} -debug=0 -loglevel=1",
        pandi.0.unwrap(),
        pandi.2.unwrap()
    ))?;

    println!("Disabled debugging");
    println!("{}", trace_output);

    // We have a full trace now so we can now
    // extract interesting stuf from it

    Ok(())
}

/// This should find ("process", "process-name", "instance number")
fn find_process_and_instance(
    s: &str,
) -> (Option<String>, Option<String>, Option<u32>) {
    // Extract the process from this:
    // omni     28848  0.0  8.6 770804 714432 ?       Sl    2018   0:50 /home/omni/bin/ibcf -i1 -ribcf_core -f/home/etc/ibcf_core.cfg -tpip=254
    let process = if let Some(idx1) = s.find("/home") {
        if let Some(idx2) = s[idx1..].find(" ") {
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
                // Just skip the -r
                Some(p[2..].to_owned().parse::<u32>().unwrap())
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

#[allow(dead_code)]
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
