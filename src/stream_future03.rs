use futures_preview::{task::Context, task::Poll};
use futures_timer::Delay;
use futures_preview::{Stream,TryStream};
use std::{pin::Pin, time::{Duration, Instant}};
use futures_preview::{prelude::*, future::{self, Either}};
use futures::Stream as _ ;

/// Information about a slot.
pub struct SlotInfo {
    /// The slot number.
    pub number: u64,
    /// Current timestamp.
    pub timestamp: u64,
    /// The instant at which the slot ends.
    pub ends_at: Instant,
    /// Slot duration.
    pub duration: u64,
}

pub struct Slots {
    last_slot: u64,
    slot_duration: u64,
    inner_delay: Option<Delay>,
}

pub enum Error {
    ReadFail,
}
use std::time::{SystemTime, UNIX_EPOCH};
impl Stream for Slots {
    type Item = Result<SlotInfo, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            let slot_duration = self.slot_duration;

            if let Some(ref mut inner_delay) = self.inner_delay {
                match Future::poll(Pin::new(inner_delay), cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(Error::ReadFail))),
                    Poll::Ready(Ok(())) => {}
                }
            }

            // timeout has fired.

            // reschedule delay for next slot.
            let ends_in = Duration::from_millis(1000);
            let ends_at = Instant::now() + ends_in;
            self.inner_delay = Some(Delay::new(ends_in));
            let slot_num = 10;
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            println!("stamp:{}",timestamp);
            // never yield the same slot twice.
            if slot_num > self.last_slot {
                self.last_slot = slot_num;

                break Poll::Ready(Some(Ok(SlotInfo {
                    number: slot_num,
                    duration: self.slot_duration,
                    timestamp,
                    ends_at,
                })))
            }
        }
    }
}

impl Slots {
    /// Create a new `Slots` stream.
    pub fn new(
        slot_duration: u64,
    ) -> Self {
        Slots {
            last_slot: 0,
            slot_duration,
            inner_delay: None,
        }
    }
}

pub fn start_slot() -> impl Future<Output = ()> {
    Slots::new(
        16
    ).inspect_err(|e| ())
        .try_for_each(move |slot_info| {
            future::ready(Ok(()))
        }).then(|res| {

        future::ready(())
    })
}

#[test]
fn slot_test() {
    let babe = start_slot();
    let babe = babe.map(|()| Ok::<(), ()>(())).compat();

    tokio::spawn(babe);
}