use gpio_cdev::{Chip, LineRequestFlags, EventRequestFlags, EventType, AsyncLineEventHandle};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use futures::stream::StreamExt;

use crate::signal::Signal;

pub type SignalHandler = Box<dyn Fn(&[Signal]) + Send + Unpin>;

async fn retrieve_data(handler: SignalHandler) -> Result<(), gpio_cdev::errors::Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?; // static device
    let input = chip.get_line(17)?; // hardcoded pin
    let mut time: u64 = 0;
    let mut count: i32 = 0;

    let mut events = Vec::new();

    println!("Waiting for signals...");

    let mut event_src = AsyncLineEventHandle::new(input.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        "ACC",
    )?)?;

    let mut delay = time::delay_for(Duration::from_millis(50));

    loop {
        tokio::select! {
            _ = &mut delay, if events.len() > 0 => {
                // check if nothing happened for a while
                let cur_time = unsafe { libc::time(std::ptr::null_mut()) };
                if cur_time as u64 > ((time + 100000000) / 1000000000 /* ns to s */) {
                    // call handler and reset events
                    handler(&events);
                    events.clear();
                }
            }
            event = event_src.next() => {
                let event = match event {
                    Some(event) => event?,
                    None => break,
                };

                let evt_time = event.timestamp();
                if count >= 1 {
                    let dur = evt_time - time;
                    match event.event_type() {
                        EventType::RisingEdge => {
                            // ignored
                        },
                        EventType::FallingEdge => {
                            events.push(Signal::decode(dur));
                        }
                    }
                }
                count += 1;
                time = evt_time;
            }
        }
    }

    Ok(())
}

pub async fn receive_loop(mut abort_rx: mpsc::Receiver<()>, handler: SignalHandler) {
    tokio::select! {
        _ = abort_rx.recv() => {
            // we've been aborted
            return;
        }
        res = retrieve_data(handler) => {
            match res {
                Ok(_) => {},
                Err(err) => { println!("An error occured during reading: {}", err); }
            }
        }
    }
}
