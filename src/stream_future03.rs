use futures_preview::{task::Context, task::Poll};
use futures_timer::Delay;
use futures_preview::Stream;
use std::{pin::Pin, time::{Duration, Instant}};

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

impl Stream for Slots {
    type Item = Result<SlotInfo, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            let slot_duration = self.slot_duration;

            if let Some(ref mut inner_delay) = self.inner_delay {
                match Future::poll(Pin::new(inner_delay), cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(Error::FaultyTimer(err)))),
                    Poll::Ready(Ok(())) => {}
                }
            }

            // timeout has fired.

            // reschedule delay for next slot.
            let ends_in = Duration::from_millis(1000);
            let ends_at = Instant::now() + ends_in;
            self.inner_delay = Some(Delay::new(ends_in));

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

pub fn start_slot() {
    Slots::new(
        slot_duration.slot_duration(),
        inherent_data_providers,
        timestamp_extractor,
    ).inspect_err(|e| debug!(target: "slots", "Faulty timer: {:?}", e))
        .try_for_each(move |slot_info| {
            // only propose when we are not syncing.
            if sync_oracle.is_major_syncing() {
                debug!(target: "slots", "Skipping proposal slot due to sync.");
                return Either::Right(future::ready(Ok(())));
            }

            let slot_num = slot_info.number;
            let chain_head = match client.best_chain() {
                Ok(x) => x,
                Err(e) => {
                    warn!(target: "slots", "Unable to author block in slot {}. \
					no best block header: {:?}", slot_num, e);
                    return Either::Right(future::ready(Ok(())));
                }
            };

            Either::Left(worker.on_slot(chain_head, slot_info).map_err(
                |e| {
                    warn!(target: "slots", "Encountered consensus error: {:?}", e);
                }).or_else(|_| future::ready(Ok(())))
            )
        }).then(|res| {
        if let Err(err) = res {
            warn!(target: "slots", "Slots stream terminated with an error: {:?}", err);
        }
        future::ready(())
    })
}