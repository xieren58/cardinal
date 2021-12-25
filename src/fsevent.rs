use super::fsevent_flags::EventFlags;
use super::fsevent_pb as pb;

use fsevent_sys::FSEventStreamEventId;
use prost::Message;

use std::{
    ffi::{CStr, OsStr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

#[derive(Debug)]
pub struct FsEvent {
    pub path: PathBuf,
    pub flag: EventFlags,
    pub id: FSEventStreamEventId,
}

impl FsEvent {
    pub(crate) fn from_raw(path: *const i8, flag: u32, id: u64) -> Self {
        let path = unsafe { CStr::from_ptr(path) };
        let path = OsStr::from_bytes(path.to_bytes());
        let path = PathBuf::from(path);
        let flag = EventFlags::from_bits_truncate(flag);
        FsEvent { path, flag, id }
    }

    fn as_pb(&self) -> pb::RawFsEvent {
        pb::RawFsEvent {
            path: self.path.as_os_str().as_bytes().to_vec(),
            flag: self.flag.bits(),
            id: self.id,
        }
    }
}

pub fn write_events_to_bytes(events: &[FsEvent]) -> Vec<u8> {
    let events = events.iter().map(|x| x.as_pb()).collect();
    let events = pb::RawFsEvents { events };
    events.encode_to_vec()
}
