use std::io::{BufRead, Write};

#[allow(dead_code)]
fn main() {
    // 监听地址: 127.0.0.1:7878
    let listener = std::net::TcpListener::bind("127.0.0.1:7878").unwrap();
    // 单线程版本
    // for element in listener.incoming() {
    //     let tcp_stream = element.unwrap();
    //     handle_stream(tcp_stream);
    // }

    
    // 多线程版本
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_stream(stream);
        });
    }
}


// 线程池包含一组已生成的线程，它们时刻等待着接收并处理新的任务。当程序接收到新任务时，
// 它会将线程池中的一个线程指派给该任务，在该线程忙着处理时，新来的任务会交给池中剩余的线程进行处理。
// 最终，当执行任务的线程处理完后，它会被重新放入到线程池中，准备处理新任务。
//-----当然，线程池依然是较为传统的提升吞吐方法，比较新的有：
// 单线程异步 IO，例如 redis；
// 多线程异步 IO，例如 Rust 的主流 web 框架。
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    #[allow(dead_code)]
    fn new(size: usize) -> Self {
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        
        // 多生产者，单一消费者（处理器）
        let (sender, receiver) = std::sync::mpsc::channel();

        let rec_wrapper: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>> = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        for i in 0..size {
            workers.push(Worker::new(i, std::sync::Arc::clone(&rec_wrapper)));
        }

        ThreadPool { 
            workers: workers,
            sender: sender
         }
    }


    //闭包
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // 可以看出，spawn 选择 FnOnce 作为 F 闭包的特征约束，原因是闭包作为任务只需被线程执行一次即可。
        // F 还有一个特征约束 Send ，也可以照抄过来，毕竟闭包需要从一个线程传递到另一个线程，
        // 至于生命周期约束 'static，是因为我们并不知道线程需要多久时间来执行该任务。
        
        let job: Box<F> = Box::new(f);
        let did_send = self.sender.send(job);
        if let Ok(result) = did_send {

        } else {

        }
    }

    pub fn execute2<F: FnOnce() + Send + 'static>(&self, f: F) {

    }
}



/// 执行任务
struct Worker {
    id: usize,
    thread: std::thread::JoinHandle<()>,
    // 任务执行器
    // receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Job>>>

}

impl Worker {
    fn new(id: usize, receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>) -> Worker {
        let thread = std::thread::spawn(move || {
            let result = receiver.lock().unwrap().recv().unwrap();
            println!("干活啦！干活啦！");
            result(); // 执行发送过来的闭包
        });
        Worker { id: id, thread: thread }
    }
}

// 单线程的处理方案
fn handle_stream(mut stream: std::net::TcpStream) {
    //// 打印一个请求内容
    // let buf_reader = std::io::BufReader::new(&stream);
    // let lines = buf_reader.lines();
    // let lines_unwrap = lines.map(|result| result.unwrap());
    // let take_while_remove_empty = lines_unwrap.take_while(|line| line.is_empty() == false);
    // let result: Vec<_> = take_while_remove_empty.collect();
    // println!("Request {:#?}", result);

    // 给予一个应答 空白页面
    // let response = "HTTP/1.1 200 OK\r\n\r\n";
    // let response_as_bytes = response.as_bytes();
    // let size = (&stream).write(response_as_bytes).unwrap();
    // println!("响应的size = {}", size);

    // 给予一个应答 html
    // let status_line = "HTTP/1.1 200 OK";
    // let contents = std::fs::read_to_string("src/hello.html").unwrap();
    // let length = contents.len();
    // let response = format!("{status_line}\r\nContent-Length:{length}\r\n\r\n{contents}");
    // stream.write_all(response.as_bytes()).unwrap();

    // 给予一个应答 多场景适配
    let buf_reader = std::io::BufReader::new(&stream);
    // 注意迭代器方法 next 的使用，原因在于我们只需要读取第一行，判断具体的 HTTP METHOD 是什么。
    let first = buf_reader.lines().next().unwrap().unwrap();
    println!("first = {:?}", first);


    if first.starts_with("GET") {

        if first.starts_with("GET /sleep") {
            // 制造一个延迟....
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
        // 路径判断 GET / HTTP/1.1  中 / 使用的是根路径

        let status_line = "HTTP/1.1 200 OK";
        let contents = std::fs::read_to_string("src/hello.html").unwrap();
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();

    } else {

        // 别的类型的方法
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = std::fs::read_to_string("404.html").unwrap();
        let length = contents.len();

        let response = format!(
            "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}
