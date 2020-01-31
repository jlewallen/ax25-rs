use std::io;
use std::io::prelude::*;
use std::net::Shutdown;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

const FEND: u8 = 0xC0;
const FESC: u8 = 0xDB;
const TFEND: u8 = 0xDC;
const TFESC: u8 = 0xDD;

pub struct TcpKissInterface {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl TcpKissInterface {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<TcpKissInterface> {
        let stream = TcpStream::connect(addr)?;
        Ok(TcpKissInterface {
            stream,
            buffer: Vec::new(),
        })
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }

    pub fn receive_frame(&mut self) -> io::Result<Vec<u8>> {
        loop {
            if let Some(frame) = make_frame_from_buffer(&mut self.buffer) {
                return Ok(frame);
            }
            let mut buf = vec![0u8; 1024];
            let n_bytes = self.stream.read(&mut buf)?;
            self.buffer.extend(buf.iter().take(n_bytes));
        }
    }

    pub fn send_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        // 0x00 is the KISS command byte, which is two nybbles
        // port = 0
        // command = 0 (all following bytes are a data frame to transmit)
        self.stream.write_all(&[FEND, 0x00])?;
        self.stream.write_all(frame)?;
        self.stream.write_all(&[FEND])?;
        self.stream.flush()?;
        Ok(())
    }
}

fn make_frame_from_buffer(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let mut possible_frame = Vec::new();

    enum Scan {
        LookingForStartMarker,
        Data,
        Escaped,
    }
    let mut state = Scan::LookingForStartMarker;
    let mut final_idx = 0;

    // Check for possible frame read-only until we know we have a complete frame
    // If we take one out, clear out buffer up to the final index
    for (idx, &c) in buffer.iter().enumerate() {
        match state {
            Scan::LookingForStartMarker => {
                if c == FEND {
                    state = Scan::Data;
                }
            }
            Scan::Data => {
                if c == FEND {
                    if !possible_frame.is_empty() {
                        // Successfully read a non-zero-length frame
                        final_idx = idx;
                        break;
                    }
                } else if c == FESC {
                    state = Scan::Escaped;
                } else {
                    possible_frame.push(c);
                }
            }
            Scan::Escaped => {
                if c == TFEND {
                    possible_frame.push(FEND);
                } else if c == TFESC {
                    possible_frame.push(FESC);
                } else if c == FEND && !possible_frame.is_empty() {
                    // Successfully read a non-zero-length frame
                    final_idx = idx;
                    break;
                }
                state = Scan::Data;
            }
        }
    }

    match final_idx {
        0 => None,
        n => {
            // Draining up to "n" will leave the final FEND in place
            // This way we can use it as the start marker for the next frame
            buffer.drain(0..n);
            Some(possible_frame)
        }
    }
}

#[test]
fn test_normal_frame() {
    let mut rx = vec![FEND, 0x01, 0x02, FEND];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_trailing_data() {
    let mut rx = vec![FEND, 0x01, 0x02, FEND, 0x03, 0x04];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(rx, vec![FEND, 0x03, 0x04]);
}

#[test]
fn test_leading_data() {
    let mut rx = vec![0x03, 0x04, FEND, 0x01, 0x02, FEND];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_consecutive_marker() {
    let mut rx = vec![FEND, FEND, FEND, 0x01, 0x02, FEND];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_escapes() {
    let mut rx = vec![FEND, 0x01, FESC, TFESC, 0x02, FESC, TFEND, 0x03, FEND];
    assert_eq!(
        make_frame_from_buffer(&mut rx),
        Some(vec![0x01, FESC, 0x02, FEND, 0x03])
    );
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_incorrect_escape_skipped() {
    let mut rx = vec![
        FEND, 0x01, FESC, 0x04, TFESC, /* passes normally without leading FESC */
        0x02, FEND,
    ];
    assert_eq!(
        make_frame_from_buffer(&mut rx),
        Some(vec![0x01, TFESC, 0x02])
    );
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_two_frames_single_fend() {
    let mut rx = vec![FEND, 0x01, 0x02, FEND, 0x03, 0x04, FEND];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x03, 0x04]));
    assert_eq!(rx, vec![FEND]);
}

#[test]
fn test_two_frames_double_fend() {
    let mut rx = vec![FEND, 0x01, 0x02, FEND, FEND, 0x03, 0x04, FEND];
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x01, 0x02]));
    assert_eq!(make_frame_from_buffer(&mut rx), Some(vec![0x03, 0x04]));
    assert_eq!(rx, vec![FEND]);
}
