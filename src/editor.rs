#![allow(dead_code)]

use crate::buffer::Buffer;

#[derive(Default)]
pub struct Editor {
    buffers: Vec<Buffer>,
}
