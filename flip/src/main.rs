use std::{fs::File, ptr::null_mut};
use std::fs;
use std::io::{Read, Write};
use input_linux_sys::{EV_ABS, ABS_MT_SLOT, ABS_MT_POSITION_X, ABS_MT_POSITION_Y, ABS_MT_TRACKING_ID, SYN_REPORT, EV_SYN};

use libc::{input_event, timeval, usleep};

union InputEvent {
  as_event: input_event,
  as_bytes: [u8; 24],
}

impl InputEvent {
  fn new() -> Self {
    InputEvent { as_bytes: [0; 24] }
  }
}

fn timersub(a: &timeval, b: &timeval) -> timeval {
  let mut result = timeval { tv_sec: 0, tv_usec: 0 };
  result.tv_sec = a.tv_sec - b.tv_sec;
  result.tv_usec = a.tv_usec - b.tv_usec;
  if result.tv_usec < 0 {
    result.tv_sec -= 1;
    result.tv_usec += 1000000;
  }
  result
}

fn main() {
  let mut file = fs::OpenOptions::new().read(true).write(true).open("/dev/input/event2").unwrap();
  
  let mut filled_slots: u32 = 0;
  let mut current_slot = 0;  
  let mut initial_touch: timeval = timeval { tv_sec: 0, tv_usec: 0 };

  let mut x: i32 = -1;
  let mut y: i32 = -1;
  let mut diff: i32 = 0;

  let mut time_difference: timeval;

  let right: [u16; 2] = [1200, 1024];
  let left: [u16; 2] = [200, 1024];

  loop {
    // Turn event into [u8]
    let mut event: InputEvent = InputEvent::new();
    file.read(unsafe {&mut event.as_bytes }).unwrap();
    let event = unsafe { &event.as_event };
    if event.type_ != (EV_ABS as u16) { continue; };
    
    if event.code == ABS_MT_SLOT as u16 {
      if event.value > 31 {
        println!("slot number {} should never happen\n", event.value);
      }
      current_slot = event.value;
      filled_slots |= 1u32 << current_slot;
    }

    if event.code == ABS_MT_POSITION_X as u16 {
      if x == -1 { x = event.value; };
      let d = (event.value - x).abs();
      if d > diff { diff = d; };
    }
    if event.code == ABS_MT_POSITION_Y as u16 {
      if y == -1 { y = event.value; };
      let d = (event.value - y).abs();
      if d > diff { diff = d; };
    }

    if event.code == ABS_MT_TRACKING_ID as u16 {
      if event.value == -1 {
        filled_slots &= 0xFFFFFFFF ^ (1u32 << current_slot);
        if filled_slots == 0 {
          time_difference = timersub(&event.time, &initial_touch);
          if time_difference.tv_sec == 0 && time_difference.tv_usec < 200000 {
            let usec = time_difference.tv_usec;
            println!("Quick touch at ({}, {}) [diff: {}] [{} usec]\n", x, y, diff, usec);

            if diff < 8 {
              if x > 1000 && y < 400 && y >= 0 {
                unsafe { usleep(50000); };
                write_swipe(&mut file, right, left);
              } else if x < 400 && x >= 0 && y < 400 && y >= 0 {
                unsafe { usleep(50000); };
                write_swipe(&mut file, left, right);
              }
            }
          } else {
            let sec = time_difference.tv_sec;
            println!("Long touch at ({}, {}) [diff: {}] [{} sec]\n", x, y, diff, sec);
          }
          x = -1; y = -1; diff = 0;
        }
      } else {
        if !more_than_one_touch(filled_slots) {
          initial_touch = event.time;
        }
      }
    }
  }
}


fn write_event(file: &mut File, event: &InputEvent) {
  file.write(unsafe {&event.as_bytes }).unwrap();
}

fn write_abs_event(file: &mut File, code: u16, value: i32) {
  let mut time: timeval = timeval { tv_sec: 0, tv_usec: 0 };
  unsafe { libc::gettimeofday(&mut time, null_mut()); };
  let event = InputEvent { as_event: input_event { type_: EV_ABS as u16, code: code, value: value, time} };
  write_event(file, &event);
}

fn write_syn_event(file: &mut File) {
  let mut time: timeval = timeval { tv_sec: 0, tv_usec: 0 };
  unsafe { libc::gettimeofday(&mut time, null_mut()); };
  let event = InputEvent { as_event: input_event { type_: EV_SYN as u16, code: SYN_REPORT as u16, value: 1, time } };
  write_event(file, &event);
}

fn write_swipe(fd: &mut File, from: [u16; 2], to: [u16; 2]) {
  write_abs_event(fd, ABS_MT_SLOT as u16, 1096);
  write_abs_event(fd, ABS_MT_TRACKING_ID as u16, unsafe { libc::time(null_mut()) } as i32);
  write_abs_event(fd, ABS_MT_POSITION_X as u16, from[0] as i32);
  write_abs_event(fd, ABS_MT_POSITION_Y as u16, from[1] as i32);
  write_syn_event(fd);

  unsafe { usleep(50000); };

  write_abs_event(fd, ABS_MT_POSITION_X as u16, to[0] as i32);
  write_abs_event(fd, ABS_MT_POSITION_Y as u16, to[1] as i32);
  write_syn_event(fd);

  write_abs_event(fd, ABS_MT_TRACKING_ID as u16, -1);
  write_syn_event(fd);
}

fn more_than_one_touch(touch_bitmap: u32) -> bool {
  touch_bitmap & (touch_bitmap - 1) != 0
}