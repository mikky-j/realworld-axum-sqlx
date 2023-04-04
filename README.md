# Things to do before run
- Make a `.env` file: You need to make a `.env` file that has the following variables
```env
DATABASE_URL=sqlite://<name-of-database>.db
JWT_SECRET=<token-secret>
JWT_EXPIRY_DURATION=<any-amount-of-time>
```
- Make sure you have installed the `sqlx cli` and installed `sqlite3` and any drivers it may need (Google it if you don't know how to)

- Make sure you run `sqlx migrate run` before you compile the code or else `sqlx` would throw errors

- Make sure you star the repo

Thanks