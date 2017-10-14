use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::str;

use bincode;

use super::block::Block;
use super::yarn::Yarn;

#[derive(Debug)]
pub struct YarnContainer {
    bytes: VecDeque<u8>
}

impl YarnContainer {
    pub(super) fn new(bytes: Vec<u8>) -> YarnContainer {
        let mut dequeue = VecDeque::new();

        for byte in "yarn".as_bytes().into_iter() {
            dequeue.push_back(*byte);
        }

        dequeue.extend(bytes.into_iter());

        YarnContainer {
            bytes: dequeue
        }
    }

    pub fn into_yarn(self) -> Option<Yarn> {
        let bytes: Vec<_> = self.bytes.into_iter().collect();

        if let Ok(blocks) = bincode::deserialize::<Vec<Block>>(&bytes[4..]) {
            let mut yarn = Yarn::new();

            for block in blocks {
                yarn.tie_block(block);
            }

            Some(yarn)
        } else {
            None
        }
    }
}

impl Read for YarnContainer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.bytes.len());

        for i in 0..len {
            buf[i] = self.bytes.pop_front().unwrap();
        }

        Ok(len)
    }
}

impl Write for YarnContainer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let old_len = self.bytes.len();

        for byte in buf {
            self.bytes.push_back(*byte);
        }

        if old_len < 4 && self.bytes.len() >= 4 {
            let error = io::Error::new(io::ErrorKind::InvalidData, "data is not valid Yarn");

            let slice = &self.bytes.as_slices().0[0..4];
            if let Ok(string) = str::from_utf8(slice) {
                if string != "yarn" {
                    return Err(error);
                }
            } else {
                return Err(error);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
