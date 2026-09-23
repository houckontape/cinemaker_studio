use crate::Frame;
use std::sync::{Arc, Mutex};

pub type FrameBuffer = Arc<Mutex<Option<Frame>>>;
