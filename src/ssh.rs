use crate::error::{Error, TraceResult};
use std::{
    net::{SocketAddr, TcpStream},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(5);

pub struct SshClient {
    ssh_session: ssh2::Session,
}

impl SshClient {
    pub fn connect(
        addr: SocketAddr,
        username: &str,
        password: &str,
    ) -> TraceResult<Self> {
        let timeout_start = Instant::now();

        println!("Attempting connection...");
        let tcp_connection = loop {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(1)) {
                Ok(tcp) => break tcp,
                Err(e) => {
                    if timeout_start.elapsed() < TIMEOUT {
                        println!("Retrying...");
                        thread::sleep(Duration::from_secs(1));
                    } else {
                        return Err(Error::SshConnection(e));
                    }
                }
            }
        };

        let mut ssh_session =
            ssh2::Session::new().map_err(|e| Error::SshSession(e))?;

        ssh_session.set_tcp_stream(tcp_connection);
        ssh_session
            .handshake()
            .map_err(|e| Error::SshHandshake(e))?;

        ssh_session
            .userauth_password(username, password)
            .map_err(Error::SshAuthentication)?;

        Ok(Self { ssh_session })
    }

    #[allow(dead_code)]
    pub fn send_cmds(&mut self, commands: &[&str]) -> TraceResult<String> {
        use std::io::{Read, Write};

        let mut out = Vec::new();
        let mut channel = self
            .ssh_session
            .channel_session()
            .map_err(Error::SshSession)?;

        channel.shell().map_err(Error::SshChannel)?;

        channel
            .write_all(commands.join("\n").as_bytes())
            .map_err(Error::SshConnection)?;
        channel.send_eof().map_err(Error::SshChannel)?;

        while !channel.eof() {
            channel
                .read_to_end(&mut out)
                .map_err(Error::SshConnection)?;
        }

        Ok(String::from_utf8_lossy(&out).to_string())
    }

    pub fn send_cmd(&mut self, command: &str) -> TraceResult<String> {
        use std::io::Read;

        let mut out = Vec::new();
        let mut channel = self
            .ssh_session
            .channel_session()
            .map_err(Error::SshChannel)?;

        channel
            .exec(command)
            .map_err(|e| Error::Command(e, command.to_owned()))?;
        channel.send_eof().map_err(Error::SshChannel)?;

        while !channel.eof() {
            channel
                .read_to_end(&mut out)
                .map_err(Error::SshConnection)?;
        }
        Ok(String::from_utf8_lossy(&out).to_string())
    }
}
