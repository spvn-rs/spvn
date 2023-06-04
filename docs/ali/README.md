# Preliminary Performance

Tests performed using [ali](https://github.com/nakabonne/ali).

## Methodology

Request with 4 byte payload, and common ASGI app executing the following:

```py

async def app(scope, receive, send):
    await receive() # added: receive the full payload
    await send({
        'type': 'http.response.start',
        'status': 200,
        'headers': [
            (b'content-type', b'text/plain'),
        ],
    }) # send a start callback
    await send({
        'type': 'http.response.body',
        'body': b'Hello, world!',
    }) # send a body
```

_modified from uvicorn docs_

## Execution

### 50 req / s

```bash
ali http://127.0.0.1:8000 -b body --rate=50
```

### 500 req / s

```bash
ali http://127.0.0.1:8000 -b body --rate=500
```

### 1000 req / s

```bash
ali http://127.0.0.1:8000 -b body --rate=1000
```

### 5000 req / s

```bash
ali http://127.0.0.1:8000 -b body --rate=5000
```

## Commands

### uvicorn

```bash
uvicorn dotest.baz:app
```

### hypercorn

```bash
hypercorn dotest.baz:app
```

### spvn

```bash
spvn serve --target dotest.baz:app
```

## Results

![hypercorn-50](./hypercorn-50.png)
_hypercorn @ 50 reqs/s_

![hypercorn-500](./hypercorn-500.png)
_hypercorn @ 500 reqs/s_

![hypercorn-1000](./hypercorn-1000.png)
_hypercorn @ 1000 reqs/s_

![hypercorn-5000](./hypercorn-5000.png)
_hypercorn @ 5000 reqs/s (crash / ddos thresh)_

![uvicorn-50](./uvicorn-50.png)
_uvicorn @ 50 reqs/s_

![uvicorn-500](./uvicorn-500.png)
_uvicorn @ 500 reqs/s_

![uvicorn-1000](./uvicorn-1000.png)
_uvicorn @ 1000 reqs/s_

![uvicorn-5000](./uvicorn-5000.png)
_uvicorn @ 5000 reqs/s (crash / ddos thresh)_

![spvn-50](./spvn-50.png)
_spvn @ 50 reqs/s_

![spvn-500](./spvn-500.png)
_spvn @ 500 reqs/s_

![spvn-1000](./spvn-1000.png)
_spvn @ 1000 reqs/s_

![spvn-5000](./spvn-5000.png)
_spvn @ 5000 reqs/s_

![spvn-10000](./spvn-10000.png)
_spvn @ 10000 reqs/s (dropped requests but continued service <130ms P95)_
