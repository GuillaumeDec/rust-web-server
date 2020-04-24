use std::thread;
use std::fmt;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    _id: u16,
    // use Option here such that we can later move safely the thread and call join on it.
    // Safely b/c the compiler can check that we first check the Some() isn't None...
    worker_thread_handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id_: u16, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let a_thread = thread::spawn(move || loop {
            let a_message = receiver.lock().unwrap().recv().unwrap();

            match a_message {
                Message::NewJob(a_job) => {
                    println!("Worker {} got a job; executing.", id_);
                    a_job();
                    println!("Worker {} FINISHED the job;", id_);
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id_);
                    break;
                }
            }
            // the worker is constantly listening for more jobs, indefinitely
        });  // loop

        Worker {
            _id: id_,
            worker_thread_handle: Some(a_thread),
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
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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
                    workers: workers,
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
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        // will send the right # of Terminate Messages into the comm channel
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            // we can 'take' / move the thread out of the Some
            // a None is left in place in the variant, which is why this whole thing is safe,
            // b/c a new call to the if below won't do anything. So Rust is happy, it's safe.
            // If you don't use Option, Rust won't let you move the handler, maybe b/c there's a risk
            // it gets moved twice? (somewhere else in the code?)
            if let Some(worker_thread_handle_) = worker.worker_thread_handle.take() {
                worker_thread_handle_.join().unwrap();
            }
        }
    }
}
