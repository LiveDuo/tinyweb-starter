FROM rust

COPY . .

EXPOSE 8080:8080

RUN cargo build -p client --target wasm32-unknown-unknown -r
RUN cargo build -p server -r

CMD [ "./target/release/server" ]
