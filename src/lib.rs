use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
};

use http::{HeaderName, HeaderValue, Request, Response, StatusCode};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool. The size should be grater than 0.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(4);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // whereってなに
        // Fってwhereの中で宣言できるってこと？
        // FnOnceってなんだ
        // Sendってなんだ
        // 'staticってなんだ
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

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }

                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
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

pub struct Server {
    router: Router,
    thread_pool: ThreadPool,
}

type Handler = fn(Request<BufReader<TcpStream>>) -> http::Response<String>;

impl Server {
    pub fn new() -> Server {
        Server {
            router: Router::new(),
            thread_pool: ThreadPool::new(4),
        }
    }

    pub fn get(&mut self, path: &str, handler: Handler) {
        self.router
            .add_route(Method::GET, path.to_string(), handler);
    }

    pub fn post(&mut self, path: &str, handler: Handler) {
        self.router
            .add_route(Method::POST, path.to_string(), handler);
    }

    pub fn serve(self) {
        let listener = TcpListener::bind("127.0.0.1:8091").unwrap();

        let shared_router = Arc::new(RwLock::new(self.router));

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let router = shared_router.clone();

            self.thread_pool.execute(move || {
                router.read().unwrap().handle_connection(stream);
            })
        }
    }
}

type Route = (Method, String);

struct Router {
    map: std::collections::HashMap<Route, Handler>,
}
use http::Method;
impl Router {
    pub fn new() -> Router {
        Router {
            map: std::collections::HashMap::new(),
        }
    }
    pub fn add_route(&mut self, method: Method, path: String, handler: Handler) {
        self.map.insert((method, path), handler);
    }
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let mut lines = buf_reader.lines();

        // request lineを処理してRequestを作成
        let binding = lines.next().unwrap().unwrap();
        let mut request_line = binding.split_whitespace();
        let method = Method::from_str(request_line.next().unwrap()).unwrap();
        let path = request_line.next().unwrap().to_string();

        let mut request = Request::builder()
            .method(method.clone())
            .uri("http://example.com".to_owned() + &path);

        // Headersを取得
        let headers = request.headers_mut().unwrap();
        loop {
            // 空行ならHeader行が終了している
            // HeaderとBodyは二行空いているので、もう一度行を読み込み、
            // 空行であることを確認してループを出る
            let l = lines.next().unwrap().unwrap();
            if l == "".to_string() {
                if lines.next().unwrap().unwrap() == "".to_string() {
                    break;
                } else {
                    panic!("irregal http request. two lines required between headers and body but only 1 found.");
                }
            }

            if let Some((key, value)) = l.split_once(": ") {
                // request.header(key, value);
                headers.append(
                    HeaderName::from_str(key.to_string().to_owned().as_str()).unwrap(),
                    HeaderValue::from_str(value.to_string().to_owned().as_str()).unwrap(),
                );
            }
        }

        // Bodyを取得
        let body = BufReader::new(&mut stream);

        let request = request.body(body).unwrap();

        // ここのcloneは取れる気がする
        let handler = self.map.get(&(method, path.clone()));

        let response = match handler {
            Some(h) => h(request),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("NOT FOUND".to_string())
                .unwrap(),
        };

        let status_line = "HTTP/1.1 ".to_owned() + response.status().as_str();
        let headers: String = response
            .headers()
            .iter()
            .map(|(k, v)| {
                let v = v.to_str().unwrap();
                format!("{k}: {v}\r\n")
            })
            .collect();
        let body: String = response.body().to_string();

        let res = format!("{status_line}\r\n{headers}\r\n{body}");

        stream.write_all(res.as_bytes()).unwrap();
    }
}
