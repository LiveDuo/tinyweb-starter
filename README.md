# TinyWeb Starter

This example shows an basic fullstack application built with *TinyWeb*.

- There are two pages, one with a todo component and a static about page
- The todo component, uses signals to demostrate how reactivity works
- When the state of the application changes it sends a request to the backend that uses the `tiny-http` crate.

## Development

Clone this repository:
```sh
git clone https://github.com/liveduo/tinyweb-starter
```
Run in development:
```sh
cargo run
```

*Note:* Restart the server to see changes.

## Production

Deploy the example using [Cloud Run](https://cloud.google.com/run) with:

```sh
docker build -t tinyweb-starter .
gcloud projects list / export PROJECT_ID=$PROJECT_ID
docker tag tinyweb-starter gcr.io/$PROJECT_ID/tinyweb-starter
docker push gcr.io/$PROJECT_ID/tinyweb-starter
gcloud run deploy $PROJECT_ID --image gcr.io/$PROJECT_ID/tinyweb-starter --project $PROJECT_ID --allow-unauthenticated
```
