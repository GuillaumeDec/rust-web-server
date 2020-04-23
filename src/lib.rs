use std::thread;
use std::fmt;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: u16,
    _join_handle: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id_: u16, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let a_thread = thread::spawn(move || {
            // the worker is constantly listening for more jobs, indefinitely
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.", id_);
                job();
                println!("Worker {} FINISHED the job;", id_);
            }
        });
        Worker {
            _id: id_,
            _join_handle: a_thread,
        }

    }
}

#[derive(Debug)] // derive std::fmt::Debug on AppError
pub struct PoolThreadError {
    code: usize,
    _message: String,
}

impl fmt::Display for PoolThreadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.code {
            1 => "Bad ThreadPool size of 0 !",
            _ => "Sorry, something is wrong! Please Try Again!",
        };
        write!(f, "{}", err_msg)
    }
}

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Result<ThreadPool, PoolThreadError> {
//        assert!(size > 0);
        match size {
            _s if size >= 1 && size <= 4 => {
                let mut workers = Vec::with_capacity(size);

                let (sender, receiver) = mpsc::channel();

                let receiver = Arc::new(Mutex::new(receiver));

                for an_id in 1..(size+1) {
                    // create some threads and store them in the vector
                    // Arc::clone will increment the # of shared ref to the mut receiver!
                    workers.push(Worker::new(an_id as u16, Arc::clone(&receiver)));
                }

                Ok(ThreadPool {
                    _workers: workers,
                    sender: sender,
                })
            },

            _s if size < 1 => Err(PoolThreadError {
                code: 1,
                _message: String::from(""),
            }),

            _ => Err(PoolThreadError {
                code: 2,
                _message: String::from("Too many threads requested"),
            }),
        }
    }
//    pub fn new(size: usize) -> ThreadPool  {
//        assert!(size > 0);
//
//        ThreadPool
//    }

    // f is a closure with three traits, one of them is a lifetime
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        // unwrap to let the case of sending fails happening, and we know it won't be happening
        self.sender.send(job).unwrap();
    }
}
