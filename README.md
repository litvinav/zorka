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

| Countdown                             | Approval                              | Blocker                               |
| ------------------------------------- | ------------------------------------- | ------------------------------------- |
| ![countdown](/docs/img/countdown.png) | ![approval](/docs/img/approval.png)   | ![blocker](/docs/img/blocker.png)     |

These client pages support light and dark mode. Since i cannot cover all variety of languages, Zorka expects you to bring your own internationalization. You can use [the default configuration](./configuration.yaml) for English or use it as reference for your language.

Countdown is displayed as a yellow clock gate on the dashboard.

Approval is displayed as a blue shield gate on the dashboard.

Blocker is displayed as a red stop sign gate on the dashboard.

### Authentication

The current idea around Zorka is to only expose `/s/:slug` routes but if necessary,
you can protect the shortcut CRUD operations and the dashboard routes via a fixed username and password.

Please refer to 'Configuration via configuration.yaml' in the examples section, if you are interested in authentication on the admin routes.

## Examples

#### Seeding with a seed.csv
```yaml
rmbl,https://rumble.com/,trusted,0,253370764861000
falcon,https://www.spacex.com/vehicles/falcon-9,untrusted,0,253370764861000
ysd,https://yandex.ru/search/?text=yandex+self+driving,trusted,0,1678406461000
zorka,https://github.com/litvinav/zorka,untrusted,1679270461000,253370764861000
```
Each line consists of the slug, a full url, trust and the two values for the availability window as two u128 values, representing milliseconds since the unix epoch. For example in JavaScript you can get the date with `new Date().getTime()`.

The slug can be any text with a length bigger than 0 up to 64.
The trust level can be currently set to 'trusted' and 'untrusted'. In case of untrusted the user has to approve his redirect and sees the URL he will be visiting.

#### Configuration via configuration.yaml
```yaml
auth:
  basic:
    username: username
    password: password
i18n:
  lang: en
  dir: ltr
  countdown: The link you want to reach will be available soon.
  blocker: The link you are trying to reach is no longer reachable.
  approval:
    label: Are you sure you want to be redirected to the following URL?
    button: to the stars 🚀
```
If you are not interested in the authentication, set the auth to none like so:

`auth: none`

#### Deployment via compose.yaml
```yaml
services:
  zorka:
    image: litvinav/zorka:latest
    environment:
      RUST_LOG: "zorka=info"
      # remove or set to not "true" to keep config
      DELETE_CONFIG: "true"
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv:ro
    # config will be deleted on launch because of DELETE_CONFIG
    - ./configuration.yaml:/app/configuration.yaml
```
If you are feeling paranoid or cannot use a amd64 image, you can always build Zorka from source and store the image in your registry.
The configuration file will be deleted on launch if DELETE_CONFIG is set to "true". This will help you to hide the the auth values in the case you are protecting the shortcut manipulation routes BUT for example 'restart: always' in docker will fail, due to a missing file.

### Preview

Admin dashboard

| Dashboard Desktop                 | Dashboard Mobile                |
| --------------------------------- | ------------------------------- |
| ![desktop](/docs/img/desktop.png) | ![mobile](/docs/img/mobile.png) |

Client pages variants

| Dark Mode                                       | Light Mode                                             |
| ----------------------------------------------- | ------------------------------------------------------ |
| ![dark mode countdown](/docs/img/countdown.png) | ![light mode countdown](/docs/img/countdown_light.png) |
