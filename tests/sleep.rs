use std::time::{Duration, Instant};

use sans_io_runtime::{Runtime, sleep};

#[test]
fn multiple() {
    let mut now = Instant::now();
    let start = now;
    let mut runtime = Runtime::default();

    runtime.spawn(
        async move {
            sleep::until(now + Duration::from_secs(2)).await;

            println!("Task slept 2 seconds");

            sleep::until(now + Duration::from_secs(4)).await;

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
