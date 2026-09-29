use std::io::{self, Read};
use std::process::ChildStdout;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(unix))]
use std::time::Duration;

/// No detached reader owns stdout: the caller reads nonblocking pipe chunks and
/// checks cancellation even while a child holds an incomplete frame open.
pub(super) struct ProcessReader {
    stdout: ChildStdout,
    #[cfg(unix)]
    wake: std::os::unix::net::UnixStream,
}

pub(super) struct ReadCancellation {
    pub(super) cancelled: AtomicBool,
    #[cfg(unix)]
    wake: std::os::unix::net::UnixStream,
}

impl ReadCancellation {
    pub(super) fn cancel(&self) {
        if !self.cancelled.swap(true, Ordering::AcqRel) {
            #[cfg(unix)]
            // Closing the wake socket wakes poll without allocating or waiting
            // for stdout or for the operating system to terminate the child.
            let _ = self.wake.shutdown(std::net::Shutdown::Both);
        }
    }
}

impl ProcessReader {
    pub(super) fn new(stdout: ChildStdout) -> io::Result<(Self, ReadCancellation)> {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let fd = stdout.as_raw_fd();
            // SAFETY: stdout owns this valid descriptor for both fcntl calls.
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags == -1
                || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        #[cfg(unix)]
        let (wake_reader, wake_writer) = std::os::unix::net::UnixStream::pair()?;
        Ok((
            Self {
                stdout,
                #[cfg(unix)]
                wake: wake_reader,
            },
            ReadCancellation {
                cancelled: AtomicBool::new(false),
                #[cfg(unix)]
                wake: wake_writer,
            },
        ))
    }

    pub(super) fn read_frame(
        &mut self,
        cancelled: &AtomicBool,
    ) -> Result<Option<Vec<u8>>, crate::protocol::ProtocolError> {
        let result = crate::protocol::read_frame(&mut CancellableRead {
            reader: self,
            cancelled,
        });
        if cancelled.load(Ordering::Acquire) {
            Ok(None)
        } else {
            result
        }
    }

    fn read_available(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows::Win32::{
                Foundation::{ERROR_BROKEN_PIPE, HANDLE},
                System::Pipes::PeekNamedPipe,
            };
            let mut available = 0;
            // SAFETY: the owned pipe handle and output pointer remain valid.
            let result = unsafe {
                PeekNamedPipe(
                    HANDLE(self.stdout.as_raw_handle()),
                    None,
                    0,
                    None,
                    Some(&mut available),
                    None,
                )
            };
            if let Err(error) = result {
                if error.code() == ERROR_BROKEN_PIPE.to_hresult() {
                    return Ok(0);
                }
                return Err(io::Error::other(error));
            }
            if available == 0 {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            let length = buffer.len().min(available as usize);
            return self.stdout.read(&mut buffer[..length]);
        }
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let mut descriptors = [
                libc::pollfd {
                    fd: self.stdout.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                },
                libc::pollfd {
                    fd: self.wake.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                },
            ];
            // SAFETY: poll borrows two owned, initialized descriptors for this call.
            let ready = unsafe { libc::poll(descriptors.as_mut_ptr(), 2, -1) };
            if ready < 0 {
                return Err(io::Error::last_os_error());
            }
            if descriptors[1].revents != 0 {
                return Ok(0);
            }
            self.stdout.read(buffer)
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = buffer;
            Err(io::ErrorKind::Unsupported.into())
        }
    }
}

struct CancellableRead<'a> {
    reader: &'a mut ProcessReader,
    cancelled: &'a AtomicBool,
}

impl Read for CancellableRead<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.cancelled.load(Ordering::Acquire) {
                return Ok(0);
            }
            match self.reader.read_available(buffer) {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    #[cfg(not(unix))]
                    std::thread::sleep(Duration::from_millis(2));
                }
                result => return result,
            }
        }
    }
}
