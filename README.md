# Album-Storage
The project aims to create a webserver to back up locally stored images in a web application with optimized image serving and the ability to catalog images per album.
It's also PoC of implementing Relay implementation with `async-graphql` library.

The long-term goal is to make a self-hosted solution for storing images.

## Features to implement:
- [ ] image upload (restrict origin but allow for mobile connection)
- [ ] uploading images in large packages
- [ ] mounting external storage volume (to work with NAS)
- [ ] tagging images
- [ ] sharing between users

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

PWD_KEY, TOKEN, database & default user credentials should be changed for any env other than local development!
