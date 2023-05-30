[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/charliermarsh/ruff/main/assets/badge/v2.json)](https://github.com/charliermarsh/ruff)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/spvn.svg?style=flat-square)](https://pypi.org/project/spvn)
[![Wheel](https://img.shields.io/pypi/wheel/spvn?style=flat-square)](https://pypi.org/project/spvn)

---

# spvn

spvn offers rust asgi bindings for python. it is in progress, contributions & development are welcome

## Project Status

Roughly in order of priority

- [✅] Integrate standard import semantics

- [🚧] PyCaller
  - [✅] (rust) Async safe integration
  - [✅] Abstract (py fn) async / sync handle
  - [🚧] Caller pool
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

  - [✅] Async threadsafe
  - [✅] Delayed py-fn call
  - [🚧] Scheduler into py

- [🚧] Live reloader
- [🚧] Websockets

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
