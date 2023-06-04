
## Commands

### uvicorn

```bash
uvicorn dotest.baz:app
```

### hypercorn

```bash
hypercorn dotest.baz:app
```

## Methodology

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
 