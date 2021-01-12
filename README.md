# url-collector
This is my take on the NASA APOD Gogo Apps challenge, implemented in
Rust. Answers to questions asked in the task description are listed at
the bottom.

# Dependencies
- openssl
- postgresql
- pkgconfig
- diesel-cli
- Rust

# Installation
Once you meet the dependencies listed above, start by populating your Postgres database:
```console
# You can also edit .env
$ DATABASE_URL=postgres://user:password@host:port/database diesel database setup
```

# Running
To compile and run in one command do:
```console
$ cargo run --release
```

# Testing
```console
$ cargo test
$ docker-compose run url-collector cargo test # If you're using docker-compose
```

# Docker setup
You can skip the Installation/running steps by using the `docker-compose` file. 
```console
# Let's make sure the DB is available before url-collector tries to run migrations
$ docker-compose start database
$ docker-compose up url-collector
```

# Place for discussion
* **Q: What if we were to change the NASA API to some other images provider?**
** A: We could replace the `apod.rs` module with code relevant to the
new image source. Likely part of the rate-limiting and concurrent job
control system could come in handy.
* **Q: What if, apart from using NASA API, we would want to have
  another microservice fetching urls from European Space Agency. How
  much code could be reused?**
** A: This question is very similar to the first one and I think the
same answer fits nicely.
* **Q: What if we wanted to add some more query params to narrow down
  lists of urls - for example, selecting only images taken by certain
  person. (field copyright in the API response)**
** A: This should be very easy to do using the PicturesParams and
ApodQuery structs. Changes across the code path should be
straightforward.
