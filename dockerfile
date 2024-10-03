
# colima start / stop
# docker tag tinyweb-starter gcr.io/$(gcloud config get-value project)/tinyweb-starter
# docker push gcr.io/$(gcloud config get-value project)/tinyweb-starter

FROM rust

COPY . .

EXPOSE 8080:8080

RUN rustup target add wasm32-unknown-unknown
RUN cargo build -p client --target wasm32-unknown-unknown -r
RUN cargo build -p server -r

CMD [ "./target/release/server" ]
