use std::str::FromStr;

use super::echo_server::{HelloReply, HelloRequest};
use bytes::Buf;

use triple::client::TripleClient;
use triple::codec::serde_codec::SerdeCodec;
use triple::invocation::*;
use triple::server::Streaming;

pub struct EchoClient {
    inner: TripleClient,
    uri: String,
}

impl Default for EchoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EchoClient {
    pub fn new() -> Self {
        Self {
            inner: TripleClient::new(),
            uri: "".to_string(),
        }
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self.inner = self
            .inner
            .with_authority(http::uri::Authority::from_str(&self.uri).unwrap());
        self
    }

    pub async fn bidirectional_streaming_echo(
        mut self,
        req: impl IntoStreamingRequest<Message = HelloRequest>,
    ) -> Result<Response<Streaming<HelloReply>>, tonic::Status> {
        let codec = SerdeCodec::<HelloRequest, HelloReply>::default();
        self.inner
            .bidi_streaming(
                req,
                codec,
                http::uri::PathAndQuery::from_static("/bidi_stream"),
            )
            .await
    }

    pub async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, tonic::Status> {
        let (_parts, body) = req.into_parts();
        let v = serde_json::to_vec(&body).unwrap();
        let req = hyper::Request::builder()
            .uri("http://".to_owned() + &self.uri.clone() + "/hello")
            .method("POST")
            .body(hyper::Body::from(v))
            .unwrap();

        println!("request: {:?}", req);
        let response = hyper::Client::builder().build_http().request(req).await;

        match response {
            Ok(v) => {
                println!("{:?}", v);
                let (_parts, body) = v.into_parts();
                let req_body = hyper::body::to_bytes(body).await.unwrap();
                let v = req_body.chunk();
                // let codec = SerdeCodec::<HelloReply, HelloRequest>::default();
                let data: HelloReply = match serde_json::from_slice(v) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()))
                    }
                };
                Ok(Response::new(data))
            }
            Err(err) => {
                println!("{}", err);
                Err(tonic::Status::new(tonic::Code::Internal, err.to_string()))
            }
        }
    }
}
