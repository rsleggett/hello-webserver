use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread};

pub struct PoolCreationError;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop { 
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker with ID {id} received job; executing");
            job();
        });
        Worker {
            id,
            thread
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>
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

        ThreadPool { workers, sender }
    }

    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        match size {
            1.. => Ok(ThreadPool::new(size)),
            0 => Err(PoolCreationError) 
        }
    }

    pub fn execute<F>(&self, f: F) 
    where 
        F: FnOnce() + Send + 'static, 
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}