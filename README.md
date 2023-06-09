# About

This is my implementation of the [Conduit API](https://github.com/gothinkster/realworld) built in [Rust](https://www.rust-lang.org/), with [Axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx).

Make sure to give this project a star if you love it!

# Building

- Clone the repository (or download it as a `.zip` file) and then set it as your working directory.

```
$ git clone https://github.com/mikky-j/realworld-axum-sqlx.git
...output omitted...

$ cd realworld-axum-sqlx
```

- Make a `.env` file in the directory with the following variables:

```env
DATABASE_URL=sqlite://<name-of-database>.db
JWT_SECRET=<token-secret>
JWT_EXPIRY_DURATION=<any-amount-of-time>
```

- Install [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli#install) for database management.

- Create the database and apply the migrations:

```
$ sqlx database create

$ sqlx migrate run
```

- Build/run the project with cargo:

```
$ cargo run --release
```

