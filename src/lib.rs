use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread};

#[derive(Debug)]
pub struct PoolCreationError;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop { 
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => { 
                    println!("Worker with ID {id} received job; executing");
                    job(); 
                },
                Err(_) => {
                    println!("Worker with ID {id} disconnected; shutting down");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread)
        }
    }
}

///
/// ThreadPool offers a way to execute jobs in a thread of a given size
/// 
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>
}

impl ThreadPool {
    ///
    /// Creates ThreadPool
    /// 
    /// # Panics
    /// 
    /// The new function will panic if size is is 0
    /// 
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in  0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    ///
    /// Builds a ThreadPool or returns a PoolCreationError
    /// 
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        match size {
            1.. => Ok(ThreadPool::new(size)),
            0 => Err(PoolCreationError) 
        }
    }

    ///
    /// Run the job on an available thread in the pool
    /// 
    pub fn execute<F>(&self, f: F) 
    where 
        F: FnOnce() + Send + 'static, 
    {
        let job = Box::new(f);

        if let Some(sender) = self.sender.as_ref() {
            sender.send(job).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker with id {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{sync::Condvar, time::Duration};

    use thread::{sleep, Thread};

    use super::*;

    #[test]
    fn should_build_thread_pool() {
        let pool = ThreadPool::build(1);

        assert!(pool.is_ok());
    }

    #[test]
    fn should_err_when_size_is_0() {
        let pool= ThreadPool::build(0);

        assert!(pool.is_err());
    }

    #[test]
    fn should_run_job_on_pool() {
        let closed = Arc::new((Mutex::new(0), Condvar::new()));
        let cloned = Arc::clone(&closed);
        let job = move || { 
            let (lock, cvar) = &*cloned;
            let mut num = lock.lock().unwrap();
            *num += 1;
            cvar.notify_one();
        };

        let pool = ThreadPool::build(1).unwrap();

        pool.execute(job);

        let (lock, cvar) = &*closed;
        let mut num = lock.lock().unwrap();
        while *num != 1 {
            num = cvar.wait(num).unwrap();
        }

        assert_eq!(*num, 1);
    }
}