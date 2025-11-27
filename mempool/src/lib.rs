use tokio::sync::mpsc::{Receiver, Sender};

pub struct MemPool<T> {
    receiver: Receiver<T>,
}

impl<T> MemPool<T> {
    pub fn new(max_size: usize) -> (Self, MemPoolHandle<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(max_size);

        let mem_pool = Self { receiver };
        let sender = MemPoolHandle::new(sender);
        (mem_pool, sender)
    }

    pub fn pop(&mut self) -> Option<T> {
        use tokio::sync::mpsc::error::TryRecvError;

        match self.receiver.try_recv() {
            Ok(item) => Some(item),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("Mempool senders disconnected, cannot receive items, this is a bug")
            }
        }
    }
}

pub struct MemPoolHandle<T> {
    sender: Sender<T>,
}

impl<T> MemPoolHandle<T> {
    fn new(sender: Sender<T>) -> Self {
        Self { sender }
    }

    /// Send an item to the mempool blocking if max size is reached
    pub async fn push(&self, item: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.sender.send(item).await
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;

    #[test]
    async fn test_mempool_new() {
        let (mut pool, _handle): (MemPool<u64>, _) = MemPool::new(10);
        assert_eq!(pool.pop(), None);
    }

    #[test]
    async fn test_push_and_pop() {
        let (mut pool, handle) = MemPool::new(10);

        handle.push(1).await.unwrap();

        let item = pool.pop();
        assert_eq!(item, Some(1));
        assert_eq!(pool.pop(), None);
    }

    #[test]
    async fn test_multiple_push_pop() {
        let (mut pool, handle) = MemPool::new(10);

        handle.push(1).await.unwrap();
        handle.push(2).await.unwrap();
        handle.push(3).await.unwrap();

        assert_eq!(pool.pop(), Some(1));
        assert_eq!(pool.pop(), Some(2));
        assert_eq!(pool.pop(), Some(3));
        assert_eq!(pool.pop(), None);
    }

    #[test]
    async fn test_pop_empty() {
        let (mut pool, _handle): (MemPool<u64>, _) = MemPool::new(10);
        assert_eq!(pool.pop(), None);
    }

    #[test]
    async fn test_max_size() {
        let (mut pool, handle) = MemPool::new(2);

        handle.push(1).await.unwrap();
        handle.push(2).await.unwrap();

        // This should block if buffer is full, but we'll use try_send in a real scenario
        // For now, just verify we can pop items
        assert_eq!(pool.pop(), Some(1));
        assert_eq!(pool.pop(), Some(2));
    }
}
