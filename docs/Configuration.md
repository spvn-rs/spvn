# Configuration

## Serve

```
Usage: spvn serve [OPTIONS] <py import>

Arguments:
  <py import>

Options:
      --bind <BIND> (bind target, ip:port format)
      --n-threads <N_THREADS> (concurrent servers)
      --cpu (use number of cpus for concurrency)
  -w, --watch (watch & reload, not implemented)
  -v, --verbose                        [env: SPVN_VERBOSE_PROC=] (enable logging, approx 75ms overhead)
      --ssl-cert-file <SSL_CERT_FILE>  [env: SPVN_SSL_CERT_FILE=] (enable tls)
      --ssl-key-file <SSL_KEY_FILE>    [env: SPVN_SSL_KEY_FILE=] (enable tls)
      --user <USER> (user, unimplemented)
      --proc-dir <PROC_DIR>            [env: PROC_DIR=] (process directory, important for FD limits on unix, unimplemented)
  -l, --lifespan (lifespan, unstable)
  -h, --help                           Print help
```
