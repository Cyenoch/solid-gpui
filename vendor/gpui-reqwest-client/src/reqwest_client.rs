// Modified by solid-gpui: use upstream Reqwest with pooled redirect policies and Tokio body polling.
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::{borrow::Cow, mem, pin::Pin, task::Poll, time::Duration};

use gpui_util::defer;

use anyhow::anyhow;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{AsyncRead, FutureExt as _, TryStreamExt as _};
use http_client::{RedirectPolicy, RequestTimeout, Url, http};
use regex::Regex;
use reqwest::{header::HeaderValue, redirect};

const DEFAULT_CAPACITY: usize = 4096;
const MAX_REDIRECT_CLIENTS: usize = 8;
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static REDACT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"key=[^&]+").unwrap());

pub struct ReqwestClient {
    clients: Mutex<VecDeque<(Option<u32>, reqwest::Client)>>,
    proxy: Option<Url>,
    user_agent: Option<HeaderValue>,
    read_timeout: Option<Duration>,
    handle: tokio::runtime::Handle,
}

impl ReqwestClient {
    /// Shared connection-management configuration for every client this type
    /// builds. `read_timeout` sets an idle timeout on each body read (see
    /// [`ReqwestClient::proxy_user_agent_and_read_timeout`]); `None` leaves
    /// reads without a timeout.
    fn builder(read_timeout: Option<Duration>) -> reqwest::ClientBuilder {
        let builder = reqwest::Client::builder()
            .tls_backend_preconfigured(http_client_tls::tls_config())
            .connect_timeout(Duration::from_secs(10))
            // Detect and drop connections that have silently gone bad on a
            // flaky path (NAT timeouts, resets) instead of reusing them. A
            // stale reused HTTP/2 connection is a common source of
            // `BadRecordMac` TLS errors against long-lived endpoints.
            .tcp_keepalive(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(30))
            .http2_keep_alive_interval(Duration::from_secs(15))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .http2_keep_alive_while_idle(true);
        match read_timeout {
            Some(read_timeout) => builder.read_timeout(read_timeout),
            None => builder,
        }
    }

    pub fn new() -> Self {
        Self::configured(None, None, None).expect("Failed to initialize HTTP client")
    }

    pub fn user_agent(agent: &str) -> anyhow::Result<Self> {
        Self::configured(None, Some(HeaderValue::from_str(agent)?), None)
    }

    pub fn proxy_and_user_agent(proxy: Option<Url>, user_agent: &str) -> anyhow::Result<Self> {
        Self::proxy_user_agent_and_read_timeout(proxy, user_agent, None)
    }

    /// Like [`ReqwestClient::proxy_and_user_agent`], but also applies a
    /// per-read idle timeout. `read_timeout` fires only after that long with no
    /// bytes received on a response body and resets on every chunk, so it
    /// aborts a silently stalled stream without disturbing a healthy one that
    /// merely goes quiet between chunks. Callers streaming long-lived responses
    /// (e.g. LLM completions, which can pause for tens of seconds during
    /// provider-side reasoning while keep-alive bytes still flow) should size
    /// it comfortably above the provider's keep-alive interval.
    ///
    /// Note: on macOS the timeout is measured against a monotonic clock that
    /// pauses during system sleep, so it does not fire from a suspend alone;
    /// callers that need prompt detection of a connection killed while
    /// suspended must re-validate the stream on wake themselves.
    pub fn proxy_user_agent_and_read_timeout(
        proxy: Option<Url>,
        user_agent: &str,
        read_timeout: Option<Duration>,
    ) -> anyhow::Result<Self> {
        Self::configured(
            proxy,
            Some(HeaderValue::from_str(user_agent)?),
            read_timeout,
        )
    }

    fn configured(
        proxy: Option<Url>,
        user_agent: Option<HeaderValue>,
        read_timeout: Option<Duration>,
    ) -> anyhow::Result<Self> {
        if let Some(proxy) = &proxy {
            anyhow::ensure!(
                matches!(
                    proxy.scheme(),
                    "http" | "https" | "socks4" | "socks4a" | "socks5" | "socks5h"
                ),
                "Unsupported proxy scheme: {}",
                proxy.scheme()
            );
        }
        let handle = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
            log::debug!("no tokio runtime found, creating one for Reqwest...");
            runtime().handle().clone()
        });
        let client = Self {
            clients: Mutex::new(VecDeque::new()),
            proxy,
            user_agent,
            read_timeout,
            handle,
        };
        client.client(Some(10))?;
        Ok(client)
    }

    /// Upstream Reqwest configures redirects per client. Reuse connection pools
    /// per policy, with a bound on the number of caller-selected redirect limits.
    fn client(&self, redirect_limit: Option<u32>) -> anyhow::Result<reqwest::Client> {
        let mut clients = self.clients.lock().unwrap();
        if let Some(index) = clients
            .iter()
            .position(|(limit, _)| *limit == redirect_limit)
        {
            let entry = clients.remove(index).unwrap();
            let client = entry.1.clone();
            clients.push_back(entry);
            return Ok(client);
        }

        let mut builder = Self::builder(self.read_timeout).redirect(match redirect_limit {
            Some(limit) => redirect::Policy::limited(limit as usize),
            None => redirect::Policy::none(),
        });
        if let Some(user_agent) = &self.user_agent {
            builder = builder.user_agent(user_agent.clone());
        }
        if let Some(proxy) = &self.proxy {
            builder = builder
                .proxy(reqwest::Proxy::all(proxy.clone())?.no_proxy(reqwest::NoProxy::from_env()));
        }
        let client = builder.build()?;
        if clients.len() == MAX_REDIRECT_CLIENTS {
            clients.pop_front();
        }
        clients.push_back((redirect_limit, client.clone()));
        Ok(client)
    }
}

pub fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            // Since we now have two executors, let's try to keep our footprint small
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to initialize HTTP client")
    })
}

// This struct is essentially a re-implementation of
// https://docs.rs/tokio-util/0.7.12/tokio_util/io/struct.ReaderStream.html
// except outside of Tokio's aegis
struct StreamReader {
    reader: Option<Pin<Box<dyn futures::AsyncRead + Send + Sync>>>,
    buf: BytesMut,
    capacity: usize,
}

impl StreamReader {
    fn new(reader: Pin<Box<dyn futures::AsyncRead + Send + Sync>>) -> Self {
        Self {
            reader: Some(reader),
            buf: BytesMut::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }
}

impl futures::Stream for StreamReader {
    type Item = std::io::Result<Bytes>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();

        let mut reader = match this.reader.take() {
            Some(r) => r,
            None => return Poll::Ready(None),
        };

        if this.buf.capacity() == 0 {
            let capacity = this.capacity;
            this.buf.reserve(capacity);
        }

        match poll_read_buf(&mut reader, cx, &mut this.buf) {
            Poll::Pending => {
                self.reader = Some(reader);

                Poll::Pending
            }
            Poll::Ready(Err(err)) => {
                self.reader = None;

                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Ok(0)) => {
                self.reader = None;
                Poll::Ready(None)
            }
            Poll::Ready(Ok(_)) => {
                let chunk = this.buf.split();
                self.reader = Some(reader);
                Poll::Ready(Some(Ok(chunk.freeze())))
            }
        }
    }
}

/// Implementation from <https://docs.rs/tokio-util/0.7.12/src/tokio_util/util/poll_buf.rs.html>
/// Specialized for this use case
fn poll_read_buf(
    io: &mut Pin<Box<dyn futures::AsyncRead + Send + Sync>>,
    cx: &mut std::task::Context<'_>,
    buf: &mut BytesMut,
) -> Poll<std::io::Result<usize>> {
    if !buf.has_remaining_mut() {
        return Poll::Ready(Ok(0));
    }

    let n = {
        let dst = buf.chunk_mut();

        // Safety: `chunk_mut()` returns a `&mut UninitSlice`, and `UninitSlice` is a
        // transparent wrapper around `[std::mem::MaybeUninit<u8>]`.
        let dst = unsafe { &mut *(dst as *mut _ as *mut [std::mem::MaybeUninit<u8>]) };
        let mut read_buf = tokio::io::ReadBuf::uninit(dst);
        let unfilled_portion = read_buf.initialize_unfilled();
        // SAFETY: Pin projection
        let io_pin = unsafe { Pin::new_unchecked(io) };
        // `futures::AsyncRead` reports the byte count as the poll's return
        // value; `read_buf.filled()` stays empty because the reader writes
        // through the initialized slice without advancing the `ReadBuf`.
        std::task::ready!(io_pin.poll_read(cx, unfilled_portion)?)
    };

    // Safety: `initialize_unfilled()` zero-initialized the entire spare
    // capacity, so the first `n` bytes are initialized no matter how many the
    // reader actually wrote, and `advance_mut` panics rather than exceeding
    // the capacity if `n` overstates the slice length.
    unsafe {
        buf.advance_mut(n);
    }

    Poll::Ready(Ok(n))
}

/// Response bodies are polled by GPUI's executor, but reqwest may create read timers.
struct RuntimeReader<R> {
    reader: Pin<Box<R>>,
    handle: tokio::runtime::Handle,
}

impl<R: AsyncRead> AsyncRead for RuntimeReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let _runtime = this.handle.enter();
        this.reader.as_mut().poll_read(cx, buffer)
    }
}

fn redact_error(mut error: reqwest::Error) -> reqwest::Error {
    if let Some(url) = error.url_mut()
        && let Some(query) = url.query()
        && let Cow::Owned(redacted) = REDACT_REGEX.replace_all(query, "key=REDACTED")
    {
        url.set_query(Some(redacted.as_str()));
    }
    error
}

impl http_client::HttpClient for ReqwestClient {
    fn proxy(&self) -> Option<&Url> {
        self.proxy.as_ref()
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.user_agent.as_ref()
    }

    fn send(
        &self,
        req: http::Request<http_client::AsyncBody>,
    ) -> futures::future::BoxFuture<
        'static,
        anyhow::Result<http_client::Response<http_client::AsyncBody>>,
    > {
        let (parts, body) = req.into_parts();

        let redirect_limit = match parts.extensions.get::<RedirectPolicy>() {
            Some(RedirectPolicy::NoFollow) => None,
            Some(RedirectPolicy::FollowLimit(limit)) => Some(*limit),
            Some(RedirectPolicy::FollowAll) => Some(100),
            None => Some(10),
        };
        let client = match self.client(redirect_limit) {
            Ok(client) => client,
            Err(error) => return futures::future::ready(Err(error)).boxed(),
        };
        let mut request_builder = client.request(parts.method, parts.uri.to_string());
        request_builder = request_builder.headers(parts.headers);
        if let Some(timeout) = parts.extensions.get::<RequestTimeout>() {
            request_builder = request_builder.timeout(timeout.0);
        }
        let request = request_builder.body(match body.0 {
            http_client::Inner::Empty => reqwest::Body::default(),
            http_client::Inner::Bytes(cursor) => cursor.into_inner().into(),
            http_client::Inner::AsyncReader(stream) => {
                reqwest::Body::wrap_stream(StreamReader::new(stream))
            }
        });

        let handle = self.handle.clone();
        async move {
            let join_handle = handle.spawn(async { request.send().await });
            let abort_handle = join_handle.abort_handle();
            let _abort_on_drop = defer(move || abort_handle.abort());

            let mut response = join_handle.await?.map_err(redact_error)?;

            let headers = mem::take(response.headers_mut());
            let mut builder = http::Response::builder()
                .status(response.status().as_u16())
                .version(response.version());
            *builder.headers_mut().unwrap() = headers;

            let bytes = response
                .bytes_stream()
                .map_err(futures::io::Error::other)
                .into_async_read();
            let body = http_client::AsyncBody::from_reader(RuntimeReader {
                reader: Box::pin(bytes),
                handle,
            });

            builder.body(body).map_err(|e| anyhow!(e))
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead as _, BufReader, Read as _, Write as _};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use futures::AsyncReadExt as _;
    use http_client::{
        AsyncBody, HttpClient, HttpRequestExt as _, Method, RedirectPolicy, Request as HttpRequest,
        Url,
    };

    use crate::ReqwestClient;

    #[test]
    fn test_redirect_policies_preserve_client_configuration() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            while matches!(
                stopped.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ) {
                assert!(Instant::now() < deadline, "HTTP test server timed out");
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("failed to accept HTTP request: {error}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut reader = BufReader::new(&mut stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut headers = String::new();
                loop {
                    let mut line = String::new();
                    assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                    if line == "\r\n" {
                        break;
                    }
                    headers.push_str(&line.to_ascii_lowercase());
                }
                assert!(headers.contains("user-agent: redirect-test\r\n"));
                drop(reader);
                let response = if request_line.starts_with("GET /redirect ") {
                    "HTTP/1.1 302 Found\r\nlocation: /done\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                } else {
                    assert!(request_line.starts_with("GET /done "));
                    "HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok"
                };
                stream.write_all(response.as_bytes()).unwrap();
            }
        });

        let client = ReqwestClient::proxy_and_user_agent(None, "redirect-test").unwrap();
        for (policy, expected_status) in [
            (Some(RedirectPolicy::NoFollow), Some(302)),
            (None, Some(200)),
            (Some(RedirectPolicy::FollowLimit(0)), None),
            (Some(RedirectPolicy::FollowAll), Some(200)),
            (Some(RedirectPolicy::NoFollow), Some(302)),
            (Some(RedirectPolicy::FollowLimit(2)), Some(200)),
        ] {
            let mut request = HttpRequest::get(format!("http://{address}/redirect"))
                .timeout(Duration::from_secs(2));
            if let Some(policy) = policy {
                request = request.follow_redirects(policy);
            }
            let response = futures::executor::block_on(
                client.send(request.body(AsyncBody::default()).unwrap()),
            );
            match expected_status {
                Some(status) => {
                    let mut response = response.unwrap();
                    assert_eq!(response.status().as_u16(), status);
                    let mut body = String::new();
                    futures::executor::block_on(response.body_mut().read_to_string(&mut body))
                        .unwrap();
                    assert_eq!(body, if status == 200 { "ok" } else { "" });
                }
                None => assert!(response.is_err(), "zero redirects must reject a redirect"),
            }
        }
        stop.send(()).unwrap();
        server.join().unwrap();
    }

    /// Regression test: `StreamReader::poll_next` used to drop the reader it
    /// `take()`s whenever the reader returned `Poll::Pending`, so the next
    /// poll reported end-of-stream and streamed request bodies were silently
    /// truncated. Readers backed by real I/O (e.g. `async_fs::File`) return
    /// `Pending` on their very first read, so their uploads sent zero bytes.
    #[test]
    fn test_streamed_body_survives_pending_reader() {
        let payload: Vec<u8> = (0..30_000usize).map(|byte| (byte % 251) as u8).collect();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let expected_payload = payload.clone();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                assert_ne!(read, 0, "client closed the connection mid-request");
                request.extend_from_slice(&buffer[..read]);
                if let Some(position) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let body_start = position + 4;
                    while request.len() - body_start < expected_payload.len() {
                        let read = stream.read(&mut buffer).unwrap();
                        assert_ne!(read, 0, "client closed the connection mid-body");
                        request.extend_from_slice(&buffer[..read]);
                    }
                    assert_eq!(&request[body_start..], &expected_payload);
                    break;
                }
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                .unwrap();
        });

        // A reader that returns `Pending` before every chunk, like a reader
        // backed by real I/O would.
        struct PendingFirstReader {
            data: std::io::Cursor<Vec<u8>>,
            ready: bool,
        }

        impl futures::AsyncRead for PendingFirstReader {
            fn poll_read(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
                buf: &mut [u8],
            ) -> std::task::Poll<std::io::Result<usize>> {
                if self.ready {
                    self.ready = false;
                    std::task::Poll::Ready(self.data.read(buf))
                } else {
                    self.ready = true;
                    cx.waker().wake_by_ref();
                    std::task::Poll::Pending
                }
            }
        }

        let reader = PendingFirstReader {
            data: std::io::Cursor::new(payload.clone()),
            ready: false,
        };

        let client = ReqwestClient::new();
        let request = HttpRequest::builder()
            .method(Method::PUT)
            .uri(format!("http://{address}/upload"))
            .header("Content-Length", payload.len().to_string())
            .body(AsyncBody::from_reader(reader))
            .unwrap();
        let response = futures::executor::block_on(client.send(request)).unwrap();
        assert!(response.status().is_success());
        server.join().unwrap();
    }

    #[test]
    fn test_request_timeout_applies_while_reading_response_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            loop {
                line.clear();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
            drop(reader);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 1\r\n\r\n")
                .unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut buffer = [0; 1];
            match stream.read(&mut buffer) {
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) => {}
                Err(error) => panic!("failed while waiting for the client to close: {error}"),
            }
        });

        let client = ReqwestClient::new();
        let request = HttpRequest::get(format!("http://{address}/"))
            .timeout(Duration::from_millis(100))
            .body(AsyncBody::default())
            .unwrap();
        let started_at = Instant::now();
        let mut response = futures::executor::block_on(client.send(request)).unwrap();
        let mut body = Vec::new();
        let result = futures::executor::block_on(response.body_mut().read_to_end(&mut body));
        assert!(result.is_err(), "the response body should time out");
        assert!(
            started_at.elapsed() < Duration::from_millis(500),
            "the request timeout should not wait for the server to close the connection"
        );
        drop(response);
        server.join().unwrap();
    }

    #[test]
    fn test_proxy_uri() {
        let client = ReqwestClient::new();
        assert_eq!(client.proxy(), None);

        let proxy = Url::parse("http://localhost:10809").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));

        let proxy = Url::parse("https://localhost:10809").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));

        let proxy = Url::parse("socks4://localhost:10808").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));

        let proxy = Url::parse("socks4a://localhost:10808").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));

        let proxy = Url::parse("socks5://localhost:10808").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));

        let proxy = Url::parse("socks5h://localhost:10808").unwrap();
        let client = ReqwestClient::proxy_and_user_agent(Some(proxy.clone()), "test").unwrap();
        assert_eq!(client.proxy(), Some(&proxy));
    }

    #[test]
    fn test_invalid_proxy_uri() {
        let proxy = Url::parse("socks://127.0.0.1:20170").unwrap();
        assert!(
            ReqwestClient::proxy_and_user_agent(Some(proxy), "test").is_err(),
            "An invalid proxy URL must not silently bypass the proxy"
        );
    }
}
