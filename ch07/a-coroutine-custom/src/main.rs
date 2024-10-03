use std::collections::VecDeque;

use future::Future;
use http::N_POLLS;

mod future;
mod http;

fn main() {
    let now = std::time::Instant::now();
    let mut requests = VecDeque::from([
        http::Http::get("/1000/HelloWorld"),
        http::Http::get("/1000/Wow"),
    ]);

    (0..100).for_each(|i| requests.push_back(http::Http::get(format!("/1000/wow-{i}").as_str())));

    // Simple queue
    while let Some(mut req) = requests.pop_front() {
        match req.poll() {
            future::PollState::Ready(r) => {
                println!("Reply: {r}");
            }
            future::PollState::NotReady => {
                requests.push_back(req);
            }
        }
    }

    println!("Done: {} polls", *N_POLLS.lock().unwrap());
    println!("Took {:?}", now.elapsed());
}
