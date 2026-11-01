//! Whether the owner of this PC is dictating, for the paired machines that ask:
//! a call on one of them mutes its microphone for as long as it lasts.
//!
//! The state is read off the same two sources `priority::local_is_busy` reads,
//! the phase and the jobs in flight, at each of their changes. Nothing here
//! keeps a copy of its own besides the last state sent to the listeners.

use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DictationState {
    /// From the press of the shortcut until the audio is taken
    pub recording: bool,
    /// The transcription of a dictation of this PC's, until its text is out
    pub transcribing: bool,
    /// When this state began, in milliseconds since the epoch; `None` while
    /// nothing has changed since launch
    pub since: Option<u64>,
}

impl DictationState {
    const IDLE: Self = Self {
        recording: false,
        transcribing: false,
        since: None,
    };
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Clone)]
pub struct DictationFeed(Arc<watch::Sender<DictationState>>);

impl Default for DictationFeed {
    fn default() -> Self {
        Self(Arc::new(watch::Sender::new(DictationState::IDLE)))
    }
}

impl DictationFeed {
    /// Take the state as `read` finds it now, and tell the listeners if it
    /// changed. `read` runs under the feed's lock, so two changes racing each
    /// other cannot go out in the wrong order.
    pub fn refresh(&self, read: impl FnOnce() -> (bool, bool)) -> bool {
        self.refresh_at(read, now_ms())
    }

    fn refresh_at(&self, read: impl FnOnce() -> (bool, bool), at: u64) -> bool {
        self.0.send_if_modified(|state| {
            let (recording, transcribing) = read();
            if state.recording == recording && state.transcribing == transcribing {
                return false;
            }
            *state = DictationState {
                recording,
                transcribing,
                since: Some(at),
            };
            true
        })
    }

    pub fn current(&self) -> DictationState {
        *self.0.borrow()
    }

    pub fn subscribe(&self) -> watch::Receiver<DictationState> {
        self.0.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_since_launch_has_no_start() {
        assert_eq!(DictationFeed::default().current(), DictationState::IDLE);
    }

    #[test]
    fn a_change_is_dated_and_told_once() {
        let feed = DictationFeed::default();
        let mut listener = feed.subscribe();

        assert!(feed.refresh_at(|| (true, false), 1_000));
        assert!(listener.has_changed().unwrap());
        assert_eq!(
            *listener.borrow_and_update(),
            DictationState { recording: true, transcribing: false, since: Some(1_000) }
        );

        assert!(!feed.refresh_at(|| (true, false), 2_000), "nothing changed");
        assert!(!listener.has_changed().unwrap());
        assert_eq!(feed.current().since, Some(1_000));

        assert!(feed.refresh_at(|| (false, true), 3_000));
        assert_eq!(
            feed.current(),
            DictationState { recording: false, transcribing: true, since: Some(3_000) }
        );
    }

    #[test]
    fn the_state_is_kept_with_nobody_listening() {
        let feed = DictationFeed::default();
        assert!(feed.refresh_at(|| (true, false), 1_000));
        assert!(feed.current().recording);
    }
}
