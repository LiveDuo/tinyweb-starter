build:
	cargo build --target wasm32-unknown-unknown -r
	cp target/wasm32-unknown-unknown/release/tinyweb_starter.wasm public/client.wasm
start:
	python3 -m http.server -d public
dev:
	make build
	make start
