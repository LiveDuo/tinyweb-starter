
# colima start / stop
# docker build -t tinyweb-starter .
# gcloud projects list / export PROJECT_ID=$PROJECT_ID
# docker tag tinyweb-starter gcr.io/$PROJECT_ID/tinyweb-starter
# docker push gcr.io/$PROJECT_ID/tinyweb-starter

FROM rust

COPY . .

EXPOSE 8080:8080

RUN rustup target add wasm32-unknown-unknown
RUN cargo build -p client --target wasm32-unknown-unknown -r
RUN cargo build -p server -r

CMD [ "./target/release/server" ]
