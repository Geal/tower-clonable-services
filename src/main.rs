use futures::future::BoxFuture;
use tower::{BoxError, Service, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let mut service = A {
        inner: B {
            data: "hello".to_string(),
            counter: 0,
        },
    };

    let req = Request { condition: false };
    let res = service.ready().await?.call(req.clone()).await?;
    println!("{:?} => {:?}", req, res);

    let req = Request { condition: true };
    let res = service.ready().await?.call(req.clone()).await?;
    println!("{:?} => {:?}", req, res);

    let req = Request { condition: false };
    let res = service.ready().await?.call(req.clone()).await?;
    println!("{:?} => {:?}", req, res);

    Ok(())
}

#[derive(Clone, Debug)]

struct Request {
    condition: bool,
}
#[derive(Clone, Debug)]

struct Response {
    value: String,
}

struct A {
    inner: B,
}

#[derive(Clone)]
struct B {
    data: String,
    counter: usize,
}

impl Service<Request> for A {
    type Response = Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // using the inner service depends on the condition here, so
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut service = self.inner.clone();
        Box::pin(async move {
            if req.condition {
                Ok(Response {
                    value: "A".to_string(),
                })
            } else {
                service.call(req).await
            }
        })
    }
}

impl Service<Request> for B {
    type Response = Response;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request) -> Self::Future {
        self.counter += 1;
        let value = format!("{}:{}", self.data, self.counter);

        Box::pin(async move { Ok(Response { value }) })
    }
}
