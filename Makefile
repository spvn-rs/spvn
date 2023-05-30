test-env:
		python3.10 -m venv ./env \
		&& . ./env/bin/activate \
		&& pip install maturin
link:
		maturin develop
.PHONY: activate
activate:
		. ./env/bin/activate
.PHONY: link-actv
link-actv: activate link
run:
		target/debug/spvn
changelog:
		git-changelog -o CHANGELOG.md
.PHONY: vars
unexport CONDA_PREFIX
vars:
		export PYO3_PYTHON=/Users/joshuaauchincloss/Movies/spvn_pyo3/env/bin/python
lint:
		black ./**/**.py &&
		ruff check . --fix &&
		cargo fmt --fix




#### perf testing things ###
uvicorn_env:
		uvicorn dotest.foo:app --port 9009
hypercorn_env:
		hypercorn dotest.baz:app --bind localhost:9010
