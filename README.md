# Webringer
![Logo](https://github.com/d4wns-l1ght/webringer/blob/main/static/logo.svg)

A crate for running a [webring](https://en.wikipedia.org/wiki/Webring). It comes with a pre-made
GUI but you can just also add it to your project and use the `ring` and `site` modules and write your own HTML/CSS (see [[#Using it as a library crate]])

## Running it as a binary
### Getting the executable

Run `cargo install webringer`

### The database
This runs on an SQLite database, with the path set in the config file (see [[#Configuration]]), defaulting to `cwd/data.db`. 
The database will be automatically created when the program is executed.

### Actually running it
Just run `webringer`! If you don't have any configuration there are sensible defaults in place.

Then, open the website (whether hosted locally, on the web, etc), log into the default admin
account (username: admin, password: admin), create a new admin account, **AND DELETE THE OLD ONE**.
Now you're ready to go! Get your friends to add their sites to the webring!

### Configuration
Configuration is read from a `.env` file, starting in the current directory and then checking all the parents.

#### Options
| Key   | Usage    | Default |
|--------------- | --------------- | --------------- |
| `DATABASE_URL`   | The location of the database, in the format `sqlite://relative/path/to/db`   | `sqlite://data.db` |
| `MIN_CONNECTIONS`   | The minimum number of connections kept in the SQL connection pool   | `5` |
| `MAX_CONNECTIONS`   | The maximum number of connections kept in the SQL connection pool   | `20` |
| `ACQUIRE_TIMEOUT_SECS`   | The maximum number of seconds to spend waiting for a connection   | `10` |
| `IDLE_TIMEOUT_SECS` | Any connection that remains in the idle queue longer than this will be closed. | `300` |


## Using it as a library crate
If you'd rather use your own html templates/css, you can use the `ring` and `site` modules and
define your own `main.rs` file. Your project should look something like this (excluding
development files like `Cargo.toml`).

```
.
├── src
│   └── main.rs
├── static
│   ├── css
│   │   └── style.css
│   ├── 404.html
│   └── logo.svg
├── templates
│   ├── admin
│   │   ├── account
│   │   │   └── change-password.html
│   │   ├── account.html
│   │   ├── add.html
│   │   ├── landing_page.html
│   │   └── sites_view.html
│   ├── base.html
│   ├── index.html
│   ├── join.html
│   ├── list.html
│   └── login.html
```
