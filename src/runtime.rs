use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

#[derive(Default)]
struct Runtime {
    // ready_queue: Arc<Mutex<Vec<usize>>>,
    tasks: HashMap<usize, Pin<Box<dyn Future<Output = ()>>>>,
    deadlines: HashMap<usize, Instant>,
    next_id: usize,
}

impl Runtime {
    pub fn spawn<F>(&mut self, future: F, now: Instant)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task_id = self.next_id;
        self.next_id += 1;

        self.poll_task(task_id, Box::pin(future), now);
    }

    pub fn poll_timeout(&self) -> Option<Instant> {
        self.deadlines.values().min().cloned()
    }

    pub fn handle_timeout(&mut self, now: Instant) {
        for (task_id, future) in self
            .deadlines
            .iter()
            .filter_map(|(task_id, deadline)| (now >= *deadline).then_some(*task_id))
            .flat_map(|task_id| Some((task_id, self.tasks.remove(&task_id)?)))
            .collect::<Vec<_>>()
        {
            self.poll_task(task_id, future, now);
        }
    }

    fn poll_task(
        &mut self,
        task_id: usize,
        future: Pin<Box<dyn Future<Output = ()>>>,
        now: Instant,
    ) {
        let waker = Arc::new(crate::Waker::new(task_id, now));

        let std_waker = std::task::Waker::from(waker.clone());

        // Store the future with its waker
        let mut future = Box::pin(future);
        let mut context = Context::from_waker(&std_waker);

        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {}
            Poll::Pending => {
                self.tasks.insert(task_id, future);

                if let Some(deadline) = waker.deadline() {
                    self.deadlines.insert(task_id, deadline);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn can_pull_multiple_futures() {
        let mut now = Instant::now();
        let start = now;
        let mut runtime = Runtime::default();

        runtime.spawn(
            async move {
                crate::sleep::until(now + Duration::from_secs(2)).await;

                println!("Task slept 2 seconds");

                crate::sleep::until(now + Duration::from_secs(4)).await;

                println!("Task slept 2 more seconds");
            },
            now,
        );

        for _ in 0..10 {
            now += Duration::from_millis(500);

            let diff = now.duration_since(start);

            println!("{diff:?} handle_timeout");
            runtime.handle_timeout(now);
        }
    }
}
