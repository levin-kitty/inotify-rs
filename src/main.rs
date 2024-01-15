#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, os::fd::RawFd};

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

fn main() {
    let fd = unsafe { bindings::inotify_init() };
    if fd < 0 {
        panic!("inotify_init failed");
    }

    let path = CString::new("/tmp").unwrap();
    let wd = unsafe {
        bindings::inotify_add_watch(
            fd,
            path.as_ptr(),
            bindings::IN_CREATE | bindings::IN_MODIFY | bindings::IN_DELETE,
        )
    };
    if wd < 0 {
        panic!("inotify_add_watch failed");
    }

    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        let length = unsafe {
            libc::read(
                fd as RawFd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            )
        };
        if length < 0 {
            panic!("read error");
        }

        let mut i = 0;
        while (i as isize) < length {
            let event = unsafe { &*(buffer.as_ptr().add(i) as *const bindings::inotify_event) };
            if event.len > 0 {
                let name = unsafe {
                    std::ffi::CStr::from_ptr(
                        buffer
                            .as_ptr()
                            .add(i + std::mem::size_of::<bindings::inotify_event>())
                            as *const libc::c_char,
                    )
                };

                println!(
                    "Event: {:?}, Name: {:?}",
                    event.mask,
                    name.to_string_lossy()
                );

                i += std::mem::size_of::<bindings::inotify_event>() + event.len as usize;
            }
        }
    }

    unsafe {
        bindings::inotify_rm_watch(fd, wd);
        libc::close(fd);
    }
}
