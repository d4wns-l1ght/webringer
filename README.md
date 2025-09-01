# Webringer
![Logo](https://github.com/d4wns-l1ght/webringer/blob/main/static/logo.svg)

A crate for running a [webring](https://en.wikipedia.org/wiki/Webring). It comes with a pre-made
GUI but you can just also add it to your project and use the `ring` module as the backend, and
write your own frontend.

## Running it
### Getting the executable

Clone this repo and run `cargo build --release`, then move the executable at
`/target/release/webringer` to somewhere on your `$PATH`.

### Getting the database
This runs on an SQLite database, generated with the migrations in the `migrations` directory of
this repo. Either generate them by downloading `sqlx` (`cargo binstall sqlx`), then running `sqlx
create database && sqlx migrate run`, or however you want to set it up (just make sure it matches
the expected schema or everything will explode).

### Actually running it
Just run `webringer` in a directory with an SQLite database called `data.db`, or in a directory
with a `.env` file specifying a different database location with the form
`sqlite://relative/path/to/db`. See how to specify a port with `webringer -h`.

Then, open the website (whether hosted locally, on the web, etc), log into the default admin
account (username: admin, password: admin), create a new admin account, **AND DELETE THE OLD ONE**.
Now you're ready to go! Get your friends to add their sites to the webring!
