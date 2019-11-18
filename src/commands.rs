use crate::error::TraceResult;
use crate::ssh::SshClient;
use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};

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
        let mut ps_command = String::from("ps aux | ");

        for g in greps.iter() {
            ps_command.push_str(&format!("grep {}", g));
        }

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
}
