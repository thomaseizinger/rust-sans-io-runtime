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
        let ext_data = crate::runtime::ExtData::from_context(cx);

        if ext_data.now() >= self.deadline {
            return Poll::Ready(());
        }

        ext_data.set_deadline(self.deadline);
        Poll::Pending
    }
}
