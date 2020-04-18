//! A simple single-threaded executor.

use std::future::Future;
use std::panic::catch_unwind;
use std::thread;

use crossbeam::channel::{unbounded, Sender};
use futures::executor;
use lazy_static::lazy_static;

type Task = async_task::Task<()>;
type JoinHandle<'a, T> = async_task::JoinHandle<'a, T, ()>;

/// Spawns a future on the executor.
fn spawn<'a, F, R>(future: F) -> JoinHandle<'a, R>
where
    F: Future<Output = R> + Send + 'a,
    R: Send + 'a,
{
    lazy_static! {
        // A channel that holds scheduled tasks.
        static ref QUEUE: Sender<Task> = {
            let (sender, receiver) = unbounded::<Task>();

            // Start the executor thread.
            thread::spawn(|| {
                for task in receiver {
                    // Ignore panics for simplicity.
                    let _ignore_panic = catch_unwind(|| task.run());
                }
            });

            sender
        };
    }

    // Create a task that is scheduled by sending itself into the channel.
    let schedule = |t| QUEUE.send(t).unwrap();
    let (task, handle) = async_task::spawn(future, schedule, ());

    // Schedule the task by sending it into the channel.
    task.schedule();

    handle
}

fn main() {

    let borrow_me = 5;

    let task = async {
        println!("Hello, world: {}!", borrow_me);
    };

    // Spawn a future and await its result.
    // executor::block_on(task);

    let handle = spawn( task );
    executor::block_on(handle);
}
