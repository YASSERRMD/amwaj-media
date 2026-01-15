# Amwaj Media Server

Real-time media server for voice agents using Rust, WebRTC, and gRPC.

## Overview

High-performance media server designed for low-latency voice interactions with AI agents.

## Features

- **WebRTC Streaming**: Handle real-time audio via WebRTC
- **Audio Processing**: VAD, noise suppression, voice isolation
- **Turn Detection**: Multi-signal fusion for accurate turn-taking
- **gRPC Interface**: Bidirectional streaming with orchestrator
- **Metrics & Tracing**: Prometheus metrics and Jaeger tracing

## Performance Targets

- Processing latency: <10ms
- Concurrent connections: 10,000+

## Getting Started

```bash
cargo build --release
cargo run -- --config config.toml
```

## License

Apache 2.0
