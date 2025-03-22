use std::any::Any;
use std::sync::{Arc, Mutex};
use std::task::Wake;
use std::time::Instant;

pub struct Waker {
    task_id: usize,

    now: Instant,
    deadline: Mutex<Option<Instant>>,
}

impl Waker {
    pub fn new(task_id: usize, now: Instant) -> Self {
        Self {
            task_id,
            now,
            deadline: Mutex::new(None),
        }
    }

    pub fn from_std_waker(std_waker: std::task::Waker) -> Option<Arc<Waker>> {
        // TODO: Figure out safety check here.

        // let data = std_waker.data() as *const Arc<dyn Any>;
        // let any = unsafe { &*data };

        // if !any.is::<Arc<Self>>() {
        //     return None;
        // }

        let waker = unsafe { Arc::from_raw(std_waker.data() as *const Waker) };
        std::mem::forget(std_waker);

        Some(waker)
    }

    pub fn now(&self) -> Instant {
        self.now
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline.lock().unwrap().clone()
    }

    pub fn set_deadline(&self, deadline: Instant) {
        self.deadline.lock().unwrap().replace(deadline);
    }
}

impl Wake for Waker {
    fn wake(self: Arc<Self>) {}

    fn wake_by_ref(self: &Arc<Self>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_cast_back_to_our_waker() {
        let value = Arc::new(Waker::new(42, Instant::now()));
        let waker = std::task::Waker::from(value);

        let our_waker_again = Waker::from_std_waker(waker).unwrap();

        // let our_waker_again = unsafe { std::mem::transmute::<_, Arc<Waker>>(our_waker_again) };

        assert_eq!(our_waker_again.task_id, 42);
    }
}
