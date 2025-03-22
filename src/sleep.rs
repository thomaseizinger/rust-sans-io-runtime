use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

pub fn until(deadline: Instant) -> Sleep {
    Sleep { deadline }
}

pub struct Sleep {
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = crate::Waker::from_std_waker(cx.waker().clone())
            .expect("only our own waker is supported");

        if waker.now() >= self.deadline {
            return Poll::Ready(());
        }

        waker.set_deadline(self.deadline);
        Poll::Pending
    }
}
