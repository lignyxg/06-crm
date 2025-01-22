use std::sync::{mpsc, Arc, Mutex};
use tonic::service::Interceptor;
use tonic::{Request, Status};
use tracing::info;

#[allow(unused)]
#[derive(Clone)]
pub struct AuthInterceptor {
    sender: mpsc::Sender<Request<()>>,
    receiver: Arc<Mutex<mpsc::Receiver<Request<()>>>>,
}

// impl Interceptor for WorkerThread {
//     fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
//         info!("interceptor on thread: {:?}", thread::current().id());
//
//         let token = request
//             .metadata()
//             .get("authorization")
//             .and_then(|v| v.to_str().ok())
//             .unwrap()
//             .to_string();
//         let mut client = self.client.clone();
//         let fut = async move {
//             client
//                 .as_mut()
//                 .unwrap()
//                 .verify(VerifyRequest { token })
//                 .await
//         }
//         .boxed();
//         let resp = self.execute(fut)?;
//
//         info!("got response: {:?}", resp);
//         request.extensions_mut().insert(resp.into_inner());
//         Ok(request)
//     }
// }

impl Drop for AuthInterceptor {
    fn drop(&mut self) {
        info!("dropping AuthInterceptor");
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        info!("AuthInterceptor call");
        self.sender.send(request).expect("Interceptor send fail");
        let resp = self
            .receiver
            .lock()
            .unwrap()
            .recv()
            .expect("Interceptor receive failed");

        Ok(resp)
    }
}

impl AuthInterceptor {
    pub fn new(
        sender: mpsc::Sender<Request<()>>,
        receiver: Arc<Mutex<mpsc::Receiver<Request<()>>>>,
    ) -> Self {
        Self { sender, receiver }
    }
}
