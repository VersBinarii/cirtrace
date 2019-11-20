use crate::error::TraceResult;
use crate::ssh::SshClient;
use regex::Regex;
use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};

const CIRPACK_PROCESSES: &[&str] = &[
    "ibcf",
    "bgcf",
    "gw_sip",
    "imstscfg",
    "hactrl",
    "stup",
    "s7pnumcfg",
    "goblin",
    "goblout",
    "transit",
    "extnti_grouper",
    "ipc2netgrouper",
];

pub struct CommandRunner(RefCell<SshClient>);

impl CommandRunner {
    pub fn new<S>(host: S, username: S, password: S) -> TraceResult<Self>
    where
        S: Into<String>,
    {
        let host = host.into().parse().expect("Failed to parse the IP address");
        let socket = SocketAddr::new(IpAddr::V4(host), 22);

        let client =
            SshClient::connect(socket, &username.into(), &password.into())?;
        Ok(Self(RefCell::new(client)))
    }
    pub fn enable_debug(&self, name: &str, instance: &str) -> TraceResult<()> {
        let _ = self.0.borrow_mut().send_cmd(&format!(
            "mgt_cscf -name={} -i{} -debug=3 -loglevel=0",
            name, instance,
        ))?;
        Ok(())
    }

    pub fn disable_debug(&self, name: &str, instance: &str) -> TraceResult<()> {
        let _ = self.0.borrow_mut().send_cmd(&format!(
            "mgt_cscf -name={} -i{} -debug=0 -loglevel=1",
            name, instance,
        ))?;
        Ok(())
    }

    pub fn get_remote_time(&self) -> TraceResult<String> {
        let remote_time = self.0.borrow_mut().send_cmd("date \"+%H:%M\"")?;
        Ok(remote_time.trim().to_owned())
    }

    pub fn get_ps_list(&self, greps: &[&str]) -> TraceResult<String> {
        let mut ps_command = String::from("ps aux |");

        for g in greps.iter() {
            ps_command.push_str(&format!("grep {} |", g));
        }
        // remove last "|"
        ps_command.pop();

        Ok(self.0.borrow_mut().send_cmd(&ps_command)?)
    }

    pub fn get_trace(
        &self,
        process_name: &str,
        start_time: &str,
    ) -> TraceResult<String> {
        Ok(self.0.borrow_mut().send_cmd(
        &format!("tail -n +$(grep -m 1 -n {1} /home/log/{0}.1 | cut -d':' -f 1) /home/log/{0}.1", process_name, start_time))?)
    }

    pub fn show_status(&self) -> TraceResult<()> {
        println!("Gathering info....");
        //let ctrl = self.0.borrow_mut().send_cmd("ctrl")?;
        //println!("ctrl: {}", ctrl);

        let hostname = self.0.borrow_mut().send_cmd("uname -n")?;

        let processes: Vec<_> = CIRPACK_PROCESSES
            .iter()
            .map(|proc| {
                (
                    proc,
                    self.0
                        .borrow_mut()
                        .send_cmd(&format!("ps aux | grep {}", proc)),
                )
            })
            .filter(|(_, r)| r.is_ok()) // get only successes
            .map(|(p, r)| (p, r.unwrap())) // unpack the results
            .collect();

        println!("{:_>1$}", "_", 5 * 15 + 6);
        println!(
            "|{3:^0$} {1:^0$} {3:^0$} {2:^0$} {3:^0$}|",
            15,
            "Hostname:",
            hostname.trim(),
            ""
        );
        println!("{:_>1$}", "_", 5 * 15 + 6);
        println!(
            "|{1:^0$}|{2:^0$}|{3:^0$}|{4:^0$}|{5:^0$}|",
            15,
            "Process owner",
            "Process #",
            "Process name",
            "Instance",
            "Instance name"
        );
        for (_n, s) in processes.iter().map(|(&p, r)| {
            (
                p,
                r.split('\n')
                    .filter(|e| !e.contains("grep"))
                    .collect::<Vec<_>>(),
            )
        }) {
            for ss in s.iter().filter(|p| !p.is_empty()) {
                let p = parse_ps_line(ss);
                println!(
                    "|{1:^0$}|{2:^0$}|{3:^0$}|{4:^0$}|{5:^0$}|",
                    15, p.0, p.1, p.2, p.3, p.4
                );
            }
        }

        println!("{:_>1$}", "_", 5 * 15 + 6);
        Ok(())
    }
}

fn parse_ps_line(line: &str) -> (String, String, String, String, String) {
    lazy_static! {
        static ref PROCESS_DETAILS: Regex = Regex::new(r"^(?P<process_owner>[[:alpha:]]*)\s*(?P<process_number>[[:digit:]]{1,5}).*.home.[[:alpha:]]*.bin.(?P<process_name>[[:alnum:],_,-]*)\s*(?:-i(?P<instance_number>[[:digit:]]*))*\s*(?:-r(?P<instance_name>[[:alpha:],_,-]*))?.*$").unwrap();
    }

    let captures = PROCESS_DETAILS
        .captures(line)
        .expect("Captures unavailable");

    let process_owner = match captures.name("process_owner") {
        Some(c) => c.as_str().to_owned(),
        None => "n/a".to_owned(),
    };
    let process_number = match captures.name("process_number") {
        Some(c) => c.as_str().to_owned(),
        None => "n/a".to_owned(),
    };
    let process_name = match captures.name("process_name") {
        Some(c) => c.as_str().to_owned(),
        None => "n/a".to_owned(),
    };
    let instance_number = match captures.name("instance_number") {
        Some(c) => c.as_str().to_owned(),
        None => "n/a".to_owned(),
    };
    let instance_name = match captures.name("instance_name") {
        Some(c) => c.as_str().to_owned(),
        None => "n/a".to_owned(),
    };

    (
        process_owner,
        process_number,
        process_name,
        instance_number,
        instance_name,
    )
}
