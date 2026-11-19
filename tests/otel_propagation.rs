//! Checks that W3C trace context reaches outgoing HTTP requests.
//!
//! pact-cli sets the global propagator for one OpenTelemetry version (0.33).
//! pact-broker-cli's HTTP client must inject `traceparent` using that same
//! version. reqwest-tracing's own injection may be built against an older one
//! (0.32 at the time of writing), whose propagator is never set. If injection
//! ever moves to the wrong version, context propagation stops without any
//! error. This test drives the real binary against a fake broker and asserts that:
//!
//! 1. the request carries a valid `traceparent` header, and
//! 2. the trace and span ids in that header belong to a span exported by
//!    pact itself (i.e. the header continues pact's trace instead of being a
//!    fresh, disconnected one).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Starts a one-shot HTTP server that records the first request's headers
/// and answers with an empty HAL document.
fn fake_broker() -> (u16, mpsc::Receiver<Vec<(String, String)>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake broker");
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut headers = Vec::new();
            let mut line = String::new();
            // Skip the request line, then read headers up to the blank line.
            let _ = reader.read_line(&mut line);
            loop {
                line.clear();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    break;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
                }
            }
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/hal+json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
            );
            let _ = tx.send(headers);
        }
    });
    (port, rx)
}

#[test]
fn traceparent_continues_pact_trace() {
    let (port, rx) = fake_broker();

    let output = Command::new(env!("CARGO_BIN_EXE_pact"))
        .args([
            "--log-level",
            "info",
            "--enable-otel",
            "--otel-exporter",
            "stdout",
            "broker",
            "list-latest-pact-versions",
            "--broker-base-url",
            &format!("http://127.0.0.1:{port}"),
        ])
        .env_remove("OTEL_TRACES_EXPORTER")
        .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
        .output()
        .expect("run pact binary");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let headers = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("pact did not send a request to the fake broker");
    let traceparent = headers
        .iter()
        .find(|(k, _)| k == "traceparent")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("no traceparent header sent; headers were {headers:?}"));

    // version-traceid-spanid-flags
    let parts: Vec<&str> = traceparent.split('-').collect();
    assert_eq!(parts.len(), 4, "malformed traceparent: {traceparent}");
    let (trace_id, span_id) = (parts[1], parts[2]);
    assert_eq!(trace_id.len(), 32, "malformed trace id: {traceparent}");
    assert_eq!(span_id.len(), 16, "malformed span id: {traceparent}");
    assert_ne!(trace_id, "0".repeat(32), "invalid (zero) trace id");
    assert_ne!(span_id, "0".repeat(16), "invalid (zero) span id");

    // The stdout span exporter prints e.g. `TraceId      : <hex>` and
    // `SpanId       : <hex>`. The header must point at a span pact exported.
    let field = |name: &str, value: &str| {
        stdout.lines().any(|l| {
            l.trim()
                .strip_prefix(name)
                .map(|rest| rest.trim_start().trim_start_matches(':').trim() == value)
                .unwrap_or(false)
        })
    };
    assert!(
        field("TraceId", trace_id),
        "traceparent trace id {trace_id} not found in pact's exported spans; \
         context propagation is broken (OpenTelemetry version mismatch?)\n{stdout}"
    );
    assert!(
        field("SpanId", span_id),
        "traceparent span id {span_id} not found in pact's exported spans\n{stdout}"
    );
}
