[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/charliermarsh/ruff/main/assets/badge/v2.json)](https://github.com/charliermarsh/ruff)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/spvn.svg?style=flat-square)](https://pypi.org/project/spvn)
[![Wheel](https://img.shields.io/pypi/wheel/spvn?style=flat-square)](https://pypi.org/project/spvn)

---

# spvn

spvn is a work in progress project which seeks to bring rust asgi bindings into python. it is in progress, contributions & development are welcome

## ASGI State Management

```mermaid

stateDiagram-v2
    Request--> Bytes
    Bytes --> PyDelayed(receive)
    PyDelayed(receive) --> PyFuture(received)
    PyFuture(received) --> PythonPtr(bytes)

    PythonPtr(bytes) --> ASGI
    PythonPtr(received) --> ASGI
    PythonPtr(scope) --> ASGI

    Request --> asgi_from_request
    asgi_from_request --> PythonPtr(scope)

    Send --> SendReceiver
    Send --> PyDelayed(send)
    PyDelayed(send) --> PyFuture(sent)
    PyFuture(sent) --> None


    PyDelayed(scope) --> Caller.call
    PyDelayed(send) --> Caller.call
    PyDelayed(receive) --> Caller.call

    Caller.call --> await

    ASGI --> SendReceiver

    SendReceiver --> ReceiveStart
    SendReceiver --> ReceiveBody

    ReceiveStart --> Response
    ReceiveBody --> Response

```

## Project Status

Roughly in order of priority

- [✅] Integrate standard import semantics

- [✅] PyCaller
  - [✅] (rust) Async safe integration
  - [✅] Abstract (py fn) async / sync handle
  - [✅] Caller pool [this will be revised, its too slow]
- [🚧] Standard asgi traits & structs
  - [🚧] ASGIScope
    - [✅] (rust) Async safe integration
    - [🚧] Conversion from `tower::Body` -> `dict`
  - [✅] ASGIVersion
  - [🚧] ASGIMessage
    - [✅] Lifecycle Scope
    - [✅] HTTP Lifecycle Scope
    - [🚧] Websockets (msg integration)
- [✅] App listener
- [🚧] App dispatcher
  - [✅] Async threadsafe
  - [🚧] Lifecycle activation for caller objects
- [🚧] App scheduler

  - [✅] Injectable `awaitables` (rust ptr -> python ptr)
  - [✅] Async threadsafe
  - [✅] Delayed py-fn call
  - [🚧] Scheduler into py

- [🚧] Live reloader
- [🚧] Websockets

## Rationale & Goals

- Relieve limits by python in networking applications
  - The goal is not to create the 'fastest' ASGI server, but reliable ASGI services which don't drop calls when subject to extreme concurrency
- Safe python threadpooling unmanaged by GIL runtime

### Claims

The upper bounds of python concurrency are not <i>really</i> production ready

#### Rationale

- Uvicorn drops requests & stalls on IO > 7500 concurrent clients
- Hypercorn drops requests & stalls on IO > 7500 concurrent clients

In both, we must horizontally scale to accomodate these limits in our systems. This is further accompanied by essentially a second layer of IO bound processes, which are evidently unable to maintain highly concurrent environments

#### Proposed

Delegation of connection multiplex, stream, and IO processes into Rust, and autoinjection at runtime following standard ASGI protocol.

### Preliminary Tests

- perf has test files containing basic benchmarks
  - hypercorn @ 1 worker = 683402-788307 ns
  - spvn -> py @ 1 worker = 159201-221808 ns

This is a <i>very</i> preliminary implementation of the caller protocol using async processes.

#### Visualization

Tests performed using [ali](https://github.com/nakabonne/ali). See [perf/ali](./perf/ali/README.md) for methodology.

![spvn-5000](./perf/ali/spvn-5000.png)
_spvn @ 5000 reqs/s_

![spvn-5000](./perf/ali/spvn-10000.png)
_spvn @ 10000 reqs/s_

![uvicorn-1000](./perf/ali/uvicorn-1000.png)
_uvicorn @ 1000 reqs/s_

![uvicorn-5000](./perf/ali/uvicorn-5000.png)
_uvicorn @ 5000 reqs/s (DDOS Success)_

![hypercorn-5000](./perf/ali/hypercorn-1000.png)
_hypercorn @ 1000 reqs/s_

![hypercorn-5000](./perf/ali/hypercorn-5000.png)
_hypercorn @ 5000 reqs/s (DDOS Success)_

## Developing

### Pre-requisites

#### Python >= 3.9

1. Use virtualenv / venv

```bash
python3.10 -m (venv|virtualenv) env && \
        . ./env/bin/activate && \
        pip install maturin
```

2. Test bindings by running

```bash
maturin develop
```

#### Rust >= 1.69.0

- Build CLI

```bash
cargo build
```

- Run CLI

```bash
target/debug/spvn serve dotest.foo:app
```

## pypi

[![PyPI - Version](https://img.shields.io/pypi/v/spvn.svg?style=flat-square)](https://pypi.org/project/spvn)

-> `pip install spvn`

-> `spvn serve foo.bar:app` (dev)

## crates

| spvn          | [![Crates.io](https://img.shields.io/crates/v/spvn.svg?style=flat-square)](https://crates.io/crates/spvn)                   |
| ------------- | --------------------------------------------------------------------------------------------------------------------------- |
| spvn_caller   | [![Crates.io](https://img.shields.io/crates/v/spvn_caller.svg?style=flat-square)](https://crates.io/crates/spvn_caller)     |
| spvn_listen   | [![Crates.io](https://img.shields.io/crates/v/spvn_listen.svg?style=flat-square)](https://crates.io/crates/spvn_listen)     |
| spvn_lifespan | [![Crates.io](https://img.shields.io/crates/v/spvn_lifespan.svg?style=flat-square)](https://crates.io/crates/spvn_lifespan) |
