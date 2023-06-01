use nix::fcntl::OFlag;
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt};
use nix::errno::Errno;
use nix::unistd::dup;

use std::fs::File;
use std::io::Read;
use std::os::fd::{FromRawFd, AsRawFd};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::process::Command;

pub struct Pty {
    pub master_write_file: File,
    pub master_reciever: Receiver<u8>,

    pub slave_name: String,
}

impl Pty {
    pub fn open() -> Result<Pty, Errno> {
        let mut master = posix_openpt(OFlag::O_RDWR)?;
        grantpt(&master)?;
        unlockpt(&master)?;
        
        let slave_name = unsafe { ptsname(&master) }?;
        
        let (tx, rx) = mpsc::sync_channel(8); // sync channel has a fixed size and blocks only sender
        
        // UNIX HACKERY
        // Duping fd for new File to allow concurrent owning for writes from another thread,
        // reads are converted with thread and accesible via Receiver channel.
        // Both Fds need to be closed (as_raw_fd, opposite to into_raw_fd, does not transfer
        // ownership (and preserve file closing)).
        let master_fd = master.as_raw_fd();
        let master_duped = unsafe { File::from_raw_fd( dup(master_fd)? ) };
        
        // Convert blocking reads to non blocking channel 
        thread::spawn(move || {
            loop {
                let mut buf = [0; 1];
                master.read(&mut buf).unwrap();
                tx.send(buf[0]).unwrap();
            }
        });
        
        Ok(Pty {slave_name, master_write_file: master_duped, master_reciever: rx})
    }

    pub fn spawn_term(&self) {
        Command::new("xterm")
            .arg("-e")
            .arg(format!("python -m serial.tools.miniterm {}", self.slave_name))
            .spawn()
            .expect("Failed to spawn tty terminal");
    }
}

