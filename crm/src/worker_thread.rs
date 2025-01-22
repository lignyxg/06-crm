use auth::pb::auth_client::AuthClient;
use auth::pb::{VerifyRequest, VerifyResponse};
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::task::{Context, Poll};
use std::thread;
use std::thread::{JoinHandle, Thread};
use tokio::runtime::Runtime;
// use tokio::runtime::Runtime;
use tokio::sync::{oneshot, Mutex};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

#[allow(unused)]
#[derive(Clone)]
pub struct WorkerThread {
    rt: Arc<Option<JoinHandle<()>>>,
    msg_sender: mpsc::Sender<Message>,
    pub(crate) client: Option<AuthClient<Channel>>,
    pub thread: Arc<Option<JoinHandle<()>>>,
}

// pub type Job = Box<
//     dyn AsyncFunc<Future = Box<dyn Future<Output = Result<Response<()>, Status>> + Unpin>>
//         + Send
//         + 'static,
// >;

// pub type Job = Box<dyn Future<Output = Result<Response<VerifyResponse>, Status>> + Send + Unpin>;
pub type Job = BoxFuture<'static, Result<Response<VerifyResponse>, Status>>;

pub enum Message {
    NewReq(Job, oneshot::Sender<Response<VerifyResponse>>, u32),
    Terminate,
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::NewReq(_, _, id) => {
                write!(f, "Message::NewReq-{}", id)
            }
            Message::Terminate => write!(f, "Message::Terminate"),
        }
    }
}

impl WorkerThread {
    pub fn new(client: AuthClient<Channel>) -> Self {
        let (tx, rx) = mpsc::channel();
        info!("thread {:?} spawn worker thread", thread::current().id());

        let rt = Self::init_runtime(rx); // 启动异步运行时

        Self {
            rt: Arc::new(Some(rt)),
            msg_sender: tx, // 向异步运行时发送任务的入口
            client: Some(client),
            thread: Arc::new(None),
        }
    }

    // 连接异步运行时
    // @req_rx: 接收外部请求
    // @return: 接收结果
    pub fn setup(&mut self, req_rx: Receiver<Request<()>>) -> mpsc::Receiver<Request<()>> {
        let (res_tx, res_rx) = mpsc::channel(); // 返回内部结果
        let client = self.client.clone().unwrap();
        let sender = self.msg_sender.clone();
        let h = thread::spawn(move || {
            info!("portal thead: {:?}", thread::current().id());
            let client = client.clone();
            let sender = sender.clone();
            while let Ok(req) = req_rx.recv() {
                let resp = Self::execute2(client.clone(), sender.clone(), req);
                res_tx.send(resp).expect("worker thread send result error"); // 返回内部结果
            }
        });
        self.thread = Arc::new(Some(h));
        res_rx
    }

    fn init_runtime(rx: Receiver<Message>) -> JoinHandle<()> {
        let msg_rx: Arc<Mutex<mpsc::Receiver<Message>>> = Arc::new(Mutex::new(rx));
        thread::spawn(move || {
            let rt = Runtime::new().expect("WorkerThread Runtime build failed");
            // let rt = tokio::runtime::Builder::new_current_thread()
            //     .enable_all()
            //     .build()
            //     .expect("WorkerThread Runtime build failed");
            info!("runtime on thread: {:?}", thread::current().id());
            rt.block_on(async move {
                // 由于使用的是std::sync::mpsc::Receiver，会阻塞线程，所以需要多线程的tokio::runtime
                while let Ok(message) = msg_rx.lock().await.recv() {
                    info!(
                        "receive message: {:?} on thread: {:?}",
                        message,
                        thread::current().id()
                    );
                    match message {
                        Message::NewReq(job, task_tx, id) => {
                            info!("dispatch task {id}");
                            tokio::spawn(forward(job, task_tx)); // 任务分发
                        }
                        Message::Terminate => {
                            println!("Ending worker thread.");
                            break;
                        }
                    }
                }
                error!("Receive error?");
            });
        })
    }

    fn execute(sender: Sender<Message>, job: Job) -> Result<Response<VerifyResponse>, Status> {
        let (tx, rx) = oneshot::channel();
        let id = rand::random::<u32>() % 10000; // dummy id
        let message = Message::NewReq(job, tx, id);
        info!(
            "send message: {:?} on thread {:?}",
            message,
            thread::current().id()
        );
        sender.send(message).unwrap(); // 向异步运行时发送任务
        let f = async { rx.await.map_err(|e| Status::internal(e.to_string())) };
        let thread = ThreadWaker(thread::current());
        let waker = futures_util::task::waker(Arc::new(thread));
        let mut cx = Context::from_waker(&waker);
        info!("wait result on thread: {:?}", thread::current().id());
        futures_util::pin_mut!(f);
        loop {
            match f.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(val)) => return Ok(val),
                Poll::Ready(Err(e)) => {
                    debug!("execute err status: {:?}", e);
                    return Err(e);
                }
                Poll::Pending => {
                    debug!("thread {:?} about to park", thread::current().id());
                    // sleep(Duration::from_millis(300));
                    thread::park()
                }
            }
        }
    }

    pub fn execute2(
        mut client: AuthClient<Channel>,
        sender: Sender<Message>,
        mut req: Request<()>,
    ) -> Request<()> {
        let token = match req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
        {
            None => return req,
            Some(t) => t.split_at(4).1.trim(),
        }
        .to_string();
        let fut = async move { client.verify(VerifyRequest { token }).await }.boxed();

        match Self::execute(sender, fut) {
            Ok(resp) => {
                req.extensions_mut().insert(resp.into_inner());
            }
            Err(e) => {
                error!("verify error: {:?}", e.to_string());
            }
        }
        req
    }
}

async fn forward<Job>(job: Job, mut tx: oneshot::Sender<Response<VerifyResponse>>)
where
    Job: Future<Output = Result<Response<VerifyResponse>, Status>>,
{
    info!("forward thread: {:?}", thread::current().id());
    futures_util::pin_mut!(job);
    let res = futures_util::future::poll_fn(|cx| match job.as_mut().poll(cx) {
        Poll::Ready(val) => Poll::Ready(val),
        Poll::Pending => {
            futures_core::ready!(tx.poll_closed(cx));
            Poll::Ready(Err(Status::cancelled("request job cancelled.")))
        }
    })
    .await;
    info!(
        "forward thread: {:?}, res: {:?}",
        thread::current().id(),
        res
    );
    if let Ok(val) = res {
        let _ = tx.send(val); // 结果发出去
    }
}

pub struct ThreadWaker(Thread);

impl futures_util::task::ArcWake for ThreadWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        debug!(
            "thread {:?} about to unpark {:?}",
            thread::current().id(),
            arc_self.0.id()
        );
        arc_self.0.unpark()
    }
}

// pub trait AsyncFunc {
//     type Future: Future<Output = Result<Response<()>, Status>>;
//     fn call(&mut self, req: Request<()>) -> Self::Future;
// }

impl Drop for WorkerThread {
    fn drop(&mut self) {
        let _ = self.msg_sender.send(Message::Terminate);
        let _ = self.client.take();
    }
}
