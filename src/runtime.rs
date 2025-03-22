use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    pin::Pin,
    task::{Context, ContextBuilder, Poll},
    time::Instant,
};

#[derive(Default)]
pub struct Runtime {
    tasks: HashMap<usize, Pin<Box<dyn Future<Output = ()>>>>,
    next_id: usize,

    deadlines: HashMap<usize, Instant>,
    scheduled_datagrams: VecDeque<(SocketAddr, SocketAddr, Vec<u8>)>,
    received_datagrams: VecDeque<(SocketAddr, SocketAddr, Vec<u8>)>,
}

pub(crate) struct ExtData {
    task_id: usize,
    now: Instant,

    deadlines: HashMap<usize, Instant>,
    scheduled_datagrams: VecDeque<(SocketAddr, SocketAddr, Vec<u8>)>,
    received_datagrams: VecDeque<(SocketAddr, SocketAddr, Vec<u8>)>,
}

impl ExtData {
    pub(crate) fn from_context<'a>(cx: &'a mut Context) -> &'a mut Self {
        cx.ext()
            .downcast_mut::<crate::runtime::ExtData>()
            .expect("no `ExtData` in context, are you using the sans-IO runtime?")
    }

    pub(crate) fn now(&self) -> Instant {
        self.now
    }

    pub(crate) fn set_deadline(&mut self, deadline: Instant) {
        self.deadlines.insert(self.task_id, deadline);
    }

    pub(crate) fn buffer_udp_transmit(&mut self, src: SocketAddr, dst: SocketAddr, bytes: Vec<u8>) {
        self.scheduled_datagrams.push_back((src, dst, bytes));
    }

    pub(crate) fn take_datagram_by_src_and_dst(
        &mut self,
        src: SocketAddr,
        dst: SocketAddr,
    ) -> Option<Vec<u8>> {
        Some(self.take_datagram(src, Some(dst))?.2)
    }

    pub(crate) fn take_datagram_by_src(
        &mut self,
        src: SocketAddr,
    ) -> Option<(SocketAddr, Vec<u8>)> {
        let (_, from, datagram) = self.take_datagram(src, None)?;

        Some((from, datagram))
    }

    fn take_datagram(
        &mut self,
        src: SocketAddr,
        dst: Option<SocketAddr>,
    ) -> Option<(SocketAddr, SocketAddr, Vec<u8>)> {
        let pos = self
            .received_datagrams
            .iter()
            .position(|(local, remote, _)| *local == src && dst.is_none_or(|dst| *remote == dst))?;

        self.received_datagrams.remove(pos)
    }
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

    pub fn handle_input(
        &mut self,
        local: SocketAddr,
        remote: SocketAddr,
        msg: Vec<u8>,
        now: Instant,
    ) {
        self.received_datagrams.push_back((local, remote, msg));

        self.tick(now);
    }

    pub fn handle_timeout(&mut self, now: Instant) {
        self.tick(now);
    }

    pub fn tick(&mut self, now: Instant) {
        for (task_id, future) in std::mem::take(&mut self.tasks) {
            self.poll_task(task_id, future, now);
        }
    }

    pub fn is_finished(&self) -> bool {
        self.tasks.is_empty()
    }

    fn poll_task(
        &mut self,
        task_id: usize,
        future: Pin<Box<dyn Future<Output = ()>>>,
        now: Instant,
    ) {
        let mut ext_data = ExtData {
            now,
            task_id,
            deadlines: std::mem::take(&mut self.deadlines),
            scheduled_datagrams: std::mem::take(&mut self.scheduled_datagrams),
            received_datagrams: std::mem::take(&mut self.received_datagrams),
        };
        let mut context = ContextBuilder::from_waker(std::task::Waker::noop())
            .ext(&mut ext_data)
            .build();

        let mut future = Box::pin(future);

        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {}
            Poll::Pending => {
                self.tasks.insert(task_id, future);
            }
        }

        self.deadlines = ext_data.deadlines;
        self.scheduled_datagrams = ext_data.scheduled_datagrams;
        self.received_datagrams = ext_data.received_datagrams;
    }
}
