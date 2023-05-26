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