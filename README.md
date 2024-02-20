# Album-Storage
Project aims to create webserver to backup locally stored images in web application with optimised image serving and ability to catalog images per album.
It's also PoC of implementing Relay implementation with `async-graphql` library
Long goal it to make self hosted solution for storing images.

## Tech stack
- Rust
- Axum - web framework
- actix-web - graphql server
- actix-web-relay - for relay implementation
- postgresql with diesel.rs - database & orm
- Lust - image server

## 
- start DB:
```
docker compose up -d db1
```
- start Lust:
```
docker compose up -d lust
```

## Info
- default port:
3000

- lust console:
http://localhost:8000/ui

- album-storage graphql playground:
http://localhost:{port}/graphql

## Note

PWD_KEY, TOKEN, database & default user credentials should be changed for any env other then local development!
