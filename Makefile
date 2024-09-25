build:
	cargo build -p client --target wasm32-unknown-unknown -r
	cp target/wasm32-unknown-unknown/release/client.wasm public/client.wasm
start:
	python3 -m http.server -d public
dev:
	make build
	make start
