use std::{marker::PhantomData, pin::Pin};

use futures::{
    future::{BoxFuture, Either},
    Future,
};
use tower::{BoxError, Service, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let mut service = A {
        inner: B {
            data: "hello".to_string(),
            counter: 0,
            phantom: PhantomData,
        },
        phantom: PhantomData,
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

struct A<'a, 'b> {
    inner: B<'b>,
    phantom: PhantomData<&'a [u8]>,
}

struct B<'b> {
    data: String,
    counter: usize,
    phantom: PhantomData<&'b [u8]>,
}

impl<'a, 'b> Service<Request> for A<'a, 'b> {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'a>>;
    //type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // using the inner service depends on the condition here, but
        // we will ignore the hairy cases here for now
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let res = if req.condition {
            Either::Left(async {
                Ok(Response {
                    value: "A".to_string(),
                })
            })
        } else {
            Either::Right(self.inner.call(req))
        };
        Box::pin(res)
    }
}

impl<'b> Service<Request> for B<'b> {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

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
