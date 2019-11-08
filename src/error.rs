#[derive(Debug)]
pub enum Error {
    SshConnection(std::io::Error),
    SshSession(ssh2::Error),
    SshChannel(ssh2::Error),
    SshHandshake(ssh2::Error),
    SshAuthentication(ssh2::Error),
    Command(ssh2::Error, String),
    File(std::io::Error, std::path::PathBuf),
    Write(std::io::Error),
}

pub type TraceResult<T, E = Error> = std::result::Result<T, E>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SshConnection(e) => {
                write!(f, "Failed to connect to remote host: {}", e)
            }
            Error::SshHandshake(e) => {
                write!(f, "Failed to perform SSH handshake: {}", e)
            }
            Error::SshSession(e) => {
                write!(f, "Failed to create SSH session: {}", e)
            }
            Error::SshAuthentication(e) => {
                write!(f, "Failed to perform SSH authentication: {}", e)
            }
            Error::SshChannel(e) => {
                write!(f, "Error operating on SSH channel: {}", e)
            }
            Error::Command(e, cmd) => {
                write!(f, "Failed to execute command: {}. Error: {}", cmd, e)
            }
            Error::File(e, path) => write!(
                f,
                "Failed to open file [{}]. Error: {}",
                path.display(),
                e
            ),
            Error::Write(e) => {
                write!(f, "Failed to write to file. Error: {}", e)
            }
        }
    }
}

impl std::error::Error for Error {}
