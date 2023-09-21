use std::{
    error::Error,
    fmt,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone)]
pub struct ThreadPoolCreationError;

impl Error for ThreadPoolCreationError {}

impl fmt::Display for ThreadPoolCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "size of ThreadPool cannot be zero")
    }
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing...");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; sutting down...");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a thread pool
    ///
    /// # Example
    /// ```
    /// use http_server::ThreadPool;
    ///
    /// let pool = ThreadPool::build(4)?;
    ///
    /// for stream in listener.incoming() {
    ///     let stream = stream.unwrap();
    ///
    ///     pool.execute(|| {
    ///         // do something
    ///     });
    /// }
    /// ```
    pub fn build(size: usize) -> Result<ThreadPool, ThreadPoolCreationError> {
        if size == 0 {
            return Err(ThreadPoolCreationError);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Pass a closure to be handled by the pool
    ///
    /// # Example
    /// ```
    /// use http_server::ThreadPool;
    ///
    /// let pool = ThreadPool::build(4)?;
    ///
    /// for stream in listener.incoming() {
    ///     let stream = stream.unwrap();
    ///
    ///     pool.execute(|| {
    ///         // do something
    ///     });
    /// }
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
