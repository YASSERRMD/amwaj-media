# Amwaj Media Server

Real-time media server for voice agents using Rust, WebRTC, and gRPC.

## Overview

Amwaj Media Server is a high-performance media server designed for low-latency voice interactions with AI agents. It provides real-time audio processing, voice activity detection, and turn-taking detection for seamless conversational experiences.

## Features

- **WebRTC Streaming**: Real-time audio via WebRTC with RTP packet handling
- **Audio Processing**: Voice Activity Detection (VAD), noise suppression, voice isolation
- **Turn Detection**: Multi-signal fusion for accurate turn-taking with hysteresis
- **gRPC Interface**: Bidirectional streaming with Python/AI orchestrator
- **Metrics & Tracing**: Prometheus metrics, latency tracking, and distributed tracing

## Performance Targets

- Processing latency: <10ms
- Concurrent connections: 10,000+
- Packet loss resilience: Graceful degradation

## Project Structure

```
amwaj-media/
├── src/
│   ├── audio/          # Audio processing (VAD, features, isolation)
│   ├── detection/      # Turn detection engine
│   ├── grpc/           # gRPC server and service
│   ├── metrics/        # Prometheus metrics and latency tracking
│   └── webrtc/         # WebRTC, RTP, jitter buffer, codec
├── protos/             # gRPC protocol definitions
├── tests/              # Integration tests
└── examples/           # Usage examples
```

## Getting Started

### Prerequisites

- Rust 1.70+
- Protobuf compiler (protoc)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release -- --config config.toml
```

### Run Tests

```bash
cargo test --all
```

## Configuration

See `config.toml` for all available configuration options:

```toml
[server]
host = "0.0.0.0"
port = 50051

[audio]
sample_rate = 16000
channels = 1
frame_duration_ms = 20

[detection]
vad_sensitivity = 0.6
min_turn_duration_ms = 250
max_silence_duration_ms = 400

[metrics]
prometheus_port = 9090
```

## Architecture

```
┌──────────────────────────┐
│    WebRTC Clients        │
└────────────┬─────────────┘
             │ WebRTC (RTP/SRTP)
             ▼
┌──────────────────────────┐
│  AMWAJ MEDIA SERVER      │
│  ├─ WebRTC I/O Layer     │
│  ├─ Audio Processing     │
│  ├─ Turn Detection       │
│  ├─ gRPC Interface       │
│  └─ Metrics & Tracing    │
└──────────────┬───────────┘
               │ gRPC Streaming
               ▼
┌──────────────────────────┐
│   AI Orchestrator        │
└──────────────────────────┘
```

## Testing

```bash
# Run all tests
cargo test --all

# Run specific module tests
cargo test --test webrtc_tests
cargo test --test audio_tests
cargo test --test turn_detection_tests
cargo test --test grpc_tests
cargo test --test metrics_tests

# Run with logging
RUST_LOG=debug cargo test

# Generate coverage
cargo tarpaulin --out Html
```

## Metrics

Prometheus metrics are exposed at `http://localhost:9090/metrics`:

- `amwaj_active_connections` - Active WebRTC connections
- `amwaj_rtp_packets_received_total` - Total RTP packets received
- `amwaj_audio_frames_processed_total` - Audio frames processed
- `amwaj_processing_latency_ms` - Processing latency histogram
- `amwaj_turn_events_detected_total` - Turn events detected
- `amwaj_barge_ins_total` - Barge-in events detected

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --all`
5. Submit a pull request

## License

Apache 2.0

## Roadmap

- [ ] Full ONNX model integration for voice isolation
- [ ] Actual Opus codec integration
- [ ] STUN/TURN server support
- [ ] Distributed session management
- [ ] Kubernetes deployment configs
