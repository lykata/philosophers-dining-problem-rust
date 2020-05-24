extern crate futures;
extern crate rand;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time;

use futures::future::join_all;
use rand::Rng;
use tokio::task;

type Forks = Vec<Fork>;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Status {
    Occupied,
    Free,
}

struct Fork {
    status: Mutex<Status>
}

impl Fork {
    fn new() -> Fork {
        Fork {
            status: Mutex::new(Status::Free)
        }
    }

    fn is_free(&self) -> bool {
        *self.status.lock().unwrap() == Status::Free
    }

    fn pick_up(&self) {
        let mut status = {
            *self.status.lock().unwrap()
        };
        match status {
            Status::Free => {
                *self.status.lock().unwrap() = Status::Occupied;
            }
            Status::Occupied => {
                let mut rng = rand::thread_rng();
                while status == Status::Occupied {
                    let rand_wait_time = rng.gen_range(2, 10);
                    sleep(time::Duration::from_millis(rand_wait_time));
                    status = *self.status.lock().unwrap();
                }
            }
        };
    }
    fn put_down(&self) {
        *self.status.lock().unwrap() = Status::Free;
    }
}

#[derive(Clone, Copy)]
enum PhilosopherState {
    Hungry,
    Eating,
    Thinking
}

struct Philosopher {
    number: usize,
    forks: Arc<Forks>,
    state: Mutex<PhilosopherState>
}

impl Philosopher {
    fn new(number: usize, forks: Arc<Vec<Fork>>) -> Philosopher {
        return Philosopher {
            state: Mutex::new(PhilosopherState::Thinking),
            number,
            forks,
        };
    }

    async fn think(&self) {
        println!("Philosopher {} is thinking 🤔...", self.number);
        *self.state.lock().unwrap() = PhilosopherState::Thinking;
        let mut rng = rand::thread_rng();
        sleep(time::Duration::from_secs(rng.gen_range(1, 3)))
    }

    async fn pick_up_forks(&self) -> (&Fork, &Fork) {
        let len = self.forks.len();
        *self.state.lock().unwrap() = PhilosopherState::Hungry;

        let left_fork_number = self.number;
        let right_fork_number = (self.number + 1) % len;

        println!("Philosopher {} asks for fork {} and {}",
                 self.number,
                 left_fork_number,
                 right_fork_number
        );

        let left: &Fork = &self.forks[left_fork_number];
        let right: &Fork = &self.forks[right_fork_number];

        if !left.is_free() && right.is_free() {
            println!("Philosopher {} cannot acquire both forks, waiting..", self.number)
        }
        left.pick_up();
        right.pick_up();
        return (left, right);
    }

    async fn eat(&self, left: &Fork, right: &Fork) {
        println!("Philosopher {} started eating 🍲", self.number);
        *self.state.lock().unwrap() = PhilosopherState::Eating;

        let mut rng = rand::thread_rng();
        let rand_wait_time = rng.gen_range(2, 10);
        sleep(time::Duration::from_secs(rand_wait_time));

        println!("Philosopher {} is done eating. Putting forks down ..", self.number);
        left.put_down();
        right.put_down();
    }
}

#[tokio::main]
async fn main() {
    const MAX: usize = 10; // Max Philosophers and Forks!

    let forks = (0..MAX)
        .map(|_| Fork::new())
        .collect::<Vec<Fork>>();
    let forks = Arc::new(forks);
    let mut handles = vec![];
    for number in 0..MAX {
        let forks = Arc::clone(&forks);
        let handle = task::spawn(async move {
            let philosopher = &Philosopher::new(number, forks);
            philosopher.think().await;
            let (left, right) = philosopher.pick_up_forks().await;
            philosopher.eat(left, right).await;
        });
        handles.push(handle);
    }
    join_all(handles).await;
}
