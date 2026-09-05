use std::{
    net::UdpSocket,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use slint::ComponentHandle;

pub struct InstanceClaim {
    port: u16,
    socket: UdpSocket,
}

impl InstanceClaim {
    /// Claims AzanBoki's per-user loopback endpoint. `None` means another
    /// instance already owns it and has been asked to show its window.
    pub fn claim(lock_path: &Path) -> Result<Option<Self>> {
        let port = port_for(lock_path);
        match UdpSocket::bind(("127.0.0.1", port)) {
            Ok(socket) => {
                socket
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .context("could not configure the app-open listener")?;
                Ok(Some(Self { port, socket }))
            }
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                notify_existing(port);
                Ok(None)
            }
            Err(error) => Err(error)
                .with_context(|| format!("could not claim the app endpoint on port {port}")),
        }
    }

    pub fn listen(self, window: slint::Weak<crate::MainWindow>) -> Result<InstanceListener> {
        let Self { port, socket } = self;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let worker = thread::Builder::new()
            .name("instance-listener".into())
            .spawn(move || {
                let mut message = [0u8; 8];
                while !thread_stop.load(Ordering::SeqCst) {
                    if let Ok((length, _)) = socket.recv_from(&mut message)
                        && &message[..length] == b"show"
                    {
                        let window = window.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(window) = window.upgrade() {
                                let _ = window.show();
                                window.window().request_redraw();
                            }
                        });
                    }
                }
            })
            .context("could not start the app-open listener")?;
        Ok(InstanceListener {
            port,
            stop,
            worker: Some(worker),
        })
    }
}

pub struct InstanceListener {
    port: u16,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

fn notify_existing(port: u16) {
    if let Ok(socket) = UdpSocket::bind(("127.0.0.1", 0)) {
        let _ = socket.send_to(b"show", ("127.0.0.1", port));
    }
}

impl Drop for InstanceListener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(socket) = UdpSocket::bind(("127.0.0.1", 0)) {
            let _ = socket.send_to(b"stop", ("127.0.0.1", self.port));
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn port_for(path: &Path) -> u16 {
    // Stable FNV-1a gives each OS user/config path a repeatable high port.
    let mut hash = 2_166_136_261u32;
    for byte in path.to_string_lossy().bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    45_000 + (hash % 15_000) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_port_is_stable_and_unprivileged() {
        let path = Path::new("/tmp/azanboki/test-user/instance.lock");
        let first = port_for(path);
        assert_eq!(first, port_for(path));
        assert!((45_000..60_000).contains(&first));
    }
}
