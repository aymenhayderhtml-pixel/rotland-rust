//! Frame time spike tracking, work budgeting, and pure queues.

use std::collections::VecDeque;

/// A per-frame work queue that processes items within a per-frame budget or limit.
#[allow(dead_code)]
pub struct SpikeBudgetQueue<T> {
    queue: VecDeque<T>,
    max_per_frame: usize,
}

#[allow(dead_code)]
impl<T> SpikeBudgetQueue<T> {
    pub fn new(max_per_frame: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_per_frame,
        }
    }

    pub fn push(&mut self, item: T) {
        self.queue.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Takes up to `max_per_frame` items for processing this frame.
    pub fn take_frame_budget(&mut self) -> Vec<T> {
        let count = self.queue.len().min(self.max_per_frame);
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.queue.pop_front() {
                out.push(item);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spike_budget_queue_limits_per_frame() {
        let mut q = SpikeBudgetQueue::new(2);
        q.push(10);
        q.push(20);
        q.push(30);
        assert_eq!(q.len(), 3);

        let f1 = q.take_frame_budget();
        assert_eq!(f1, vec![10, 20]);
        assert_eq!(q.len(), 1);

        let f2 = q.take_frame_budget();
        assert_eq!(f2, vec![30]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn test_spike_budget_empty_queue() {
        let mut q: SpikeBudgetQueue<i32> = SpikeBudgetQueue::new(5);
        assert!(q.is_empty());
        let items = q.take_frame_budget();
        assert!(items.is_empty());
    }
}
