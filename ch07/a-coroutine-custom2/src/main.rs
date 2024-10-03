use std::{collections::VecDeque, process::Output};

use future::Future;
use http::{Http, N_POLLS};

mod future;
mod http;

struct Coroutine {
    state: State,
    // stack: Stack,
}

enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

enum Stack {
    Start,
    Wait1 { reply1: String },
    Wait2 { reply1: String, reply2: String },
}

fn leaf_future(stack: Stack) -> (State, Stack) {
    match stack {
        Stack::Start => {
            println!("Starting");
            (State::Wait1(Box::new(Http::get("/1000/Hello"))), stack)
        }
        Stack::Wait1 { ref reply1 } => {
            println!("{reply1}");
            (State::Wait2(Box::new(Http::get("/500/World"))), stack)
        }
        Stack::Wait2 { ref reply2, .. } => {
            println!("{reply2}");
            (State::Resolved, stack)
        }
    }
}

fn main() {
    let now = std::time::Instant::now();

    let mut stack = Stack::Start;
    loop {
        stack = match leaf_future(stack) {
            (State::Start, _) => unreachable!(),
            (State::Wait1(mut future), _) => loop {
                if let future::PollState::Ready(result) = future.poll() {
                    break Stack::Wait1 { reply1: result };
                }
            },
            (State::Wait2(mut future), stack) => loop {
                if let future::PollState::Ready(result) = future.poll() {
                    break Stack::Wait2 {
                        reply1: if let Stack::Wait1 { reply1 } = stack {
                            reply1
                        } else {
                            unreachable!()
                        },
                        reply2: result,
                    };
                }
            },
            (State::Resolved, _) => break,
        }
    }

    println!("Done: {} polls", *N_POLLS.lock().unwrap());
    println!("Took {:?}", now.elapsed());
}
