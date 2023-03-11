<p align="center">
<img src="./docs/img/logo.jpeg" alt="logo" width="240"/>
</p>

⭐ Zorka (Cyrillic: зорка, [ˈzorkə], star in Belarusian) is a fast, self contained and minimalistic url shortener, using [SQLite](https://www.sqlite.org/) to store and [Actix](https://actix.rs/) to host the redirects.

## Usage

```sh
GET     /        # web UI
GET     /s/:slug # short url redirecting to the target
GET     /s       # get current shortcuts in the csv format
PUT     /s       # put route for new entries during runtime
DELETE  /s       # deletes entries during runtime by slug or url
GET     /health  # readiness and liveness health
```

You can scale this url shortener to your requirements via seeding, since its hosting the same url's on all instances.
Seeding on launch from a `./seed.csv` is seen in the example section.

As an alternative you can just manage the redirects via the web UI and backup the sqlite.db file.

Of course, mixed usage is also possible. Notice the `/s` route that generates the shortcuts for your version in the csv format. This could be used to create a updated `./seed.csv` from one instance and redeploy all instances. This also allows fast redeploys if you are mounting the seed.csv via a ConfigMap in a Kubernetes environment.

## Protection

### Gated routes

| Countdown                             | Blocker                               |
| ------------------------------------- | ------------------------------------- |
| ![countdown](/docs/img/countdown.png) | ![blocker](/docs/img/blocker.png)     |

These pages display multiple languages in order to increase your client reach.

### Authentication

The current idea around Zorka is to only expose `/s/:slug` routes but if necessary,
you can protect the shortcut CRUD and the dashboard routes via a fixed username and password.

This can be achieved via a environment variable with the actual value of the Basic Authorization header.

To get the header value, run this in the console of your browser. Replace username and password with actual values.
```js
console.log("Basic " + btoa("username:password"))
```
☝ This will give you the AUTH env value from the example.

## Examples

Seeding with a seed.csv
```yaml
rmbl,https://rumble.com/,0,253370764861000
ysd,https://yandex.ru/search/?text=yandex+self+driving,0,1678406461000
zorka,https://github.com/litvinav/zorka,1679270461000,253370764861000
```
Each line consists of the slug, a full url, and the two values for the availability window as two u128 values, representing milliseconds since the unix epoch. For example in JavaScript you can get the date with `new Date().getTime()`.

Compose:
```yaml
services:
  zorka:
    image: litvinav/zorka:latest
    environment:
      AUTH: "Basic dXNlcm5hbWU6cGFzc3dvcmQ="
      RUST_LOG: "zorka=info"
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv:ro
```
If you are feeling paranoid or cannot use a amd64 image, you can always build Zorka from source and store the image in your registry.

### Preview

| Dashboard Desktop                 | Dashboard Mobile                |
| --------------------------------- | ------------------------------- |
| ![desktop](/docs/img/desktop.png) | ![mobile](/docs/img/mobile.png) |
