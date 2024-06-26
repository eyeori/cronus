use std::path::PathBuf;

use nng::{Error, Message, Protocol, Socket};

use crate::CronusResult;

/// `NngIpcSocket` is a structure that represents an IPC socket using the NNG library.
/// It contains the raw socket and the address of the socket as a string.
pub struct NngIpcSocket {
    /// `raw` is the raw NNG socket.
    raw: Socket,
    /// `addr` is the address of the socket, represented as a string.
    addr: String,
}

impl NngIpcSocket {
    /// Constructs a new `NngIpcSocket` with the given protocol and path.
    ///
    /// # Arguments
    ///
    /// * `p` - A protocol that the socket will use.
    /// * `path` - A path that will be used to format the address of the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Self>` - Returns a `CronusResult` that contains the newly created `NngIpcSocket` or an error.
    pub fn new(p: Protocol, path: PathBuf) -> CronusResult<Self> {
        Ok(Self {
            raw: Socket::new(p)?,
            addr: format!("ipc://{}", path.display()),
        })
    }

    /// Constructs a new `NngIpcSocket` that listens on the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path that will be used to format the address of the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Self>` - Returns a `CronusResult` that contains the newly created `NngIpcSocket` or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to listen on the given path.
    pub fn new_listen(path: PathBuf) -> CronusResult<Self> {
        let sock = Self::new(Protocol::Rep0, path)?;
        sock.listen()?;
        Ok(sock)
    }

    /// Constructs a new `NngIpcSocket` that dials synchronously to the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path that will be used to format the address of the socket.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Self>` - Returns a `CronusResult` that contains the newly created `NngIpcSocket` or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to dial synchronously to the given path.
    pub fn new_dial_sync(path: PathBuf) -> CronusResult<Self> {
        let sock = Self::new(Protocol::Req0, path)?;
        sock.dial_sync()?;
        Ok(sock)
    }

    /// Initiates listening for connections on the `NngIpcSocket`.
    ///
    /// # Returns
    ///
    /// * `CronusResult<()>` - Returns a `CronusResult` that contains an empty tuple on success or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to listen on the address.
    pub fn listen(&self) -> CronusResult<()> {
        self.raw.listen(&self.addr).map_err(Into::into)
    }

    /// Initiates a synchronous dialing operation on the `NngIpcSocket`.
    ///
    /// # Returns
    ///
    /// * `CronusResult<()>` - Returns a `CronusResult` that contains an empty tuple on success or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to dial to the address.
    pub fn dial_sync(&self) -> CronusResult<()> {
        self.raw.dial(&self.addr).map_err(Into::into)
    }

    /// Receives a message from the `NngIpcSocket`.
    ///
    /// # Returns
    ///
    /// * `CronusResult<Message>` - Returns a `CronusResult` that contains the received message or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to receive a message.
    pub fn recv(&self) -> CronusResult<Message> {
        self.raw.recv().map_err(Into::into)
    }

    /// Sends a message through the `NngIpcSocket`.
    ///
    /// # Arguments
    ///
    /// * `msg` - A message that can be converted into a `Message` type.
    ///
    /// # Returns
    ///
    /// * `CronusResult<()>` - Returns a `CronusResult` that contains an empty tuple on success or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the socket fails to send the message.
    pub fn send<M: Into<Message>>(&self, msg: M) -> CronusResult<()> {
        self.raw.send(msg).map_err(Error::from).map_err(Into::into)
    }
}

/// Implementation of the `Drop` trait for `NngIpcSocket`.
///
/// This implementation ensures that the raw NNG socket is closed when the `NngIpcSocket` is dropped.
impl Drop for NngIpcSocket {
    /// Closes the raw NNG socket when the `NngIpcSocket` is dropped.
    fn drop(&mut self) {
        self.raw.close();
    }
}
