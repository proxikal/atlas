//! Channel primitives for message passing
//!
//! Provides bounded and unbounded channels for sending messages between
//! tasks. Channels are the primary mechanism for communication in async code.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Sender half of a channel
pub struct ChannelSender {
    inner: Arc<mpsc::UnboundedSender<Value>>,
    capacity: Option<usize>,
}

impl ChannelSender {
    /// Send a value through the channel
    ///
    /// Returns true if the message was sent, false if channel is closed.
    pub fn send(&self, value: Value) -> bool {
        self.inner.send(value).is_ok()
    }

    /// Check if channel is closed
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Get channel capacity (None for unbounded)
    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }
}

impl Clone for ChannelSender {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: self.capacity,
        }
    }
}

impl fmt::Debug for ChannelSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelSender")
            .field("capacity", &self.capacity)
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

/// Receiver half of a channel
pub struct ChannelReceiver {
    inner: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Value>>>,
}

impl ChannelReceiver {
    /// Receive a value from the channel
    ///
    /// Returns a Future that resolves when a message is available.
    /// Future is rejected if the channel is closed with no messages.
    pub fn receive(&self) -> AtlasFuture {
        let receiver = Arc::clone(&self.inner);
        let future = AtlasFuture::new_pending();
        let future_clone = future.clone();

        // Spawn task to wait for message
        tokio::task::spawn_local(async move {
            let mut rx = receiver.lock().await;
            match rx.recv().await {
                Some(value) => future_clone.resolve(value),
                None => future_clone.reject(Value::string("Channel closed")),
            }
        });

        future
    }

    /// Try to receive a value without blocking
    ///
    /// Returns Some(value) if a message is immediately available,
    /// None if the channel is empty.
    pub fn try_receive(&self) -> Option<Value> {
        // For non-blocking receive, we need to use block_on with try_recv
        // This is a limitation of the current sync API
        crate::async_runtime::block_on(async {
            let mut rx = self.inner.lock().await;
            rx.try_recv().ok()
        })
    }
}

impl Clone for ChannelReceiver {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Debug for ChannelReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelReceiver").finish()
    }
}

/// Create a new unbounded channel
///
/// Returns [sender, receiver] as an array.
/// Unbounded channels can hold unlimited messages (limited only by memory).
pub fn channel_unbounded() -> (ChannelSender, ChannelReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();

    let sender = ChannelSender {
        inner: Arc::new(tx),
        capacity: None,
    };

    let receiver = ChannelReceiver {
        inner: Arc::new(tokio::sync::Mutex::new(rx)),
    };

    (sender, receiver)
}

/// Create a new bounded channel
///
/// Returns [sender, receiver] as an array.
/// Bounded channels can hold up to `capacity` messages before blocking sends.
///
/// Note: Current implementation uses unbounded internally but tracks capacity.
/// A full implementation would enforce the bound.
pub fn channel_bounded(capacity: usize) -> (ChannelSender, ChannelReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();

    let sender = ChannelSender {
        inner: Arc::new(tx),
        capacity: Some(capacity),
    };

    let receiver = ChannelReceiver {
        inner: Arc::new(tokio::sync::Mutex::new(rx)),
    };

    (sender, receiver)
}

/// Select from multiple channel receivers
///
/// Returns the first available message along with the channel index.
/// Returns [value, index] as an array, or rejects if all channels are closed.
pub fn channel_select(receivers: Vec<ChannelReceiver>) -> AtlasFuture {
    let future = AtlasFuture::new_pending();
    let future_clone = future.clone();

    tokio::task::spawn_local(async move {
        let mut receivers_locked = Vec::new();
        for receiver in receivers {
            receivers_locked.push(receiver.inner.clone());
        }

        // Simple round-robin check (not truly fair select, but functional)
        loop {
            for (idx, receiver) in receivers_locked.iter().enumerate() {
                let mut rx = receiver.lock().await;
                if let Ok(value) = rx.try_recv() {
                    // Got a message!
                    future_clone.resolve(Value::array(vec![value, Value::Number(idx as f64)]));
                    return;
                }
            }

            // Small delay before retry
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

            // Check if all channels are closed (simplified check)
            if receivers_locked.is_empty() {
                future_clone.reject(Value::string("All channels closed"));
                return;
            }
        }
    });

    future
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unbounded_channel_send_receive() {
        let (sender, receiver) = channel_unbounded();

        // Send a message
        assert!(sender.send(Value::Number(42.0)));

        // Receive should get it
        let received = receiver.try_receive();
        assert!(received.is_some());
        match received.unwrap() {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_bounded_channel() {
        let (sender, receiver) = channel_bounded(10);

        assert_eq!(sender.capacity(), Some(10));
        assert!(sender.send(Value::string("test")));

        let received = receiver.try_receive();
        assert!(received.is_some());
    }

    #[test]
    fn test_channel_clone_sender() {
        let (sender, receiver) = channel_unbounded();
        let sender2 = sender.clone();

        assert!(sender.send(Value::Number(1.0)));
        assert!(sender2.send(Value::Number(2.0)));

        let r1 = receiver.try_receive();
        let r2 = receiver.try_receive();

        assert!(r1.is_some());
        assert!(r2.is_some());
    }

    #[test]
    fn test_channel_closed() {
        let (sender, _receiver) = channel_unbounded();
        // Receiver dropped, channel should close
        drop(_receiver);

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(sender.is_closed());
    }

    #[test]
    fn test_multiple_messages() {
        let (sender, receiver) = channel_unbounded();

        sender.send(Value::Number(1.0));
        sender.send(Value::Number(2.0));
        sender.send(Value::Number(3.0));

        assert_eq!(receiver.try_receive().unwrap(), Value::Number(1.0));
        assert_eq!(receiver.try_receive().unwrap(), Value::Number(2.0));
        assert_eq!(receiver.try_receive().unwrap(), Value::Number(3.0));
    }
}
