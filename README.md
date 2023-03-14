<p align="center">
<img src="./docs/img/logo.jpeg" alt="logo" width="240"/>
</p>

⭐ Zorka (Cyrillic: зорка, [ˈzorkə], star in Belarusian) is a fast, self contained and minimalistic url shortener, using [SQLite](https://www.sqlite.org/) to store and [Actix](https://actix.rs/) to host the redirects.

## General

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

### Privacy Policy

Zorka does not store any of your or the customers data. On one side you can safely use Zorka without leaking any of your private information to other Comapnies. Also your customers wont be greeted with any anoying cookie or privacy banners.

Unless you protect the routes with [Gates](#gated-routes), your clients wont even know it is there.

## Protection

### Gated routes

These client pages support light and dark mode.

| Countdown                             | Approval                              | Blocker                               |
| ------------------------------------- | ------------------------------------- | ------------------------------------- |
| ![countdown](/docs/img/countdown.png) | ![approval](/docs/img/approval.png)   | ![blocker](/docs/img/blocker.png)     |

<p>
<svg fill="#ffd700" class="inline-block" fill="currentColor" focusable="false" aria-hidden="true" viewBox="0 0 24 24" height="20" width="20">
<path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zm3.3 14.71L11 12.41V7h2v4.59l3.71 3.71-1.42 1.41z"></path>
</svg>
Countdown - behaives like a waiting lobby for not yet ready targets. Will reload the page after the countdown hits 0.
</p>
<p>
<svg fill="#4169e1" class="inline-block" focusable="false" aria-hidden="true" viewBox="0 0 24 24" height="20" width="20"><path d="M12 2 4 5v6.09c0 5.05 3.41 9.76 8 10.91 4.59-1.15 8-5.86 8-10.91V5l-8-3zm-1.06 13.54L7.4 12l1.41-1.41 2.12 2.12 4.24-4.24 1.41 1.41-5.64 5.66z"></path>
</svg>
Approval - the URL is considered as 'untrusted' and the user should confirm the redirect.
</p>
<p>
<svg fill="#e14148" class="inline-block" focusable="false" aria-hidden="true" viewBox="0 0 24 24" height="20" width="20">
<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11H7v-2h10v2z"></path>
</svg>
Blocker - the redirect timed out and the redirect target is not reachable.
</p>

Since i cannot cover all variety of languages, Zorka expects you to bring your own internationalization. You can use [the default configuration](./configuration.yaml) for English or use it as reference for your language.

### Authentication

The current idea around Zorka is to only expose `/s/:slug` routes but if necessary,
you can protect the shortcut CRUD operations and the dashboard routes.

Please refer to [Configuration via configuration.yaml](#deployment-via-composeyaml) in the examples section, if you are interested in authentication on the admin routes.

#### Why does Zorka not provide authentication for shortcuts?

Authentication only makes sense when attached to the system it protects. This means Auth should be implemented by the resource you are redirecting to. If the resource you are redirecting to is not protected, it is unprotected. Please do not do "security through obscurity", even if you choose against using Zorka.

## Examples

### Seeding with a seed.csv
```yaml
rmbl,https://rumble.com/,trusted,0,253370764861000
falcon,https://www.spacex.com/vehicles/falcon-9,untrusted,0,253370764861000
ysd,https://yandex.ru/search/?text=yandex+self+driving,trusted,0,1678406461000
zorka,https://github.com/litvinav/zorka,untrusted,1679270461000,253370764861000
```
Each line consists of the slug, a full url, trust and the two values for the availability window as two u128 values, representing milliseconds since the unix epoch. For example in JavaScript you can get the date with `new Date().getTime()`.

The slug can be any text with a length bigger than 0 up to 64 characters.
The trust level can be currently set to 'trusted' and 'untrusted'. In case of untrusted the user has to approve his redirect and sees the URL he will be visiting.

### Configuration via configuration.yaml

>Option 3: Oauth2 Code Flow has only been tested with GitHub so far. Most OIDC flows are very similar but use it with caution. Expect other and improved Oauth2 Flows in the future when they are ready.

```yaml
auth: # Pick one option
  # Option 1: No Authentication
  none
  # Option 2: Basic Authentication
  basic:
    username: username
    password: password
  # Option 3: SSO via Oauth2 Code Flow
  oauth2:
    client_id: d675f5571ab33e44bb32
    client_secret: fb27917c2d78f59ef49f38e6454dad675f55718s
    scopes: user offline
    auth_url: https://github.com/login/oauth/authorize
    token_url: https://github.com/login/oauth/access_token
    introspect_url: https://api.github.com/user
    redirect_url: http://localhost:8080/oauth2/code
i18n:
  lang: en
  dir: ltr
  countdown: The link you want to reach will be available soon.
  blocker: The link you are trying to reach is no longer reachable.
  approval:
    label: Are you sure you want to be redirected to the following URL?
    button: to the stars 🚀
```

### Deployment via compose.yaml
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
The configuration file will be deleted on launch if DELETE_CONFIG is set to "true".

This will help you to hide the the auth values in the case you are protecting the shortcut manipulation routes BUT for example 'restart: always' in docker will fail, due to a missing file.

### Preview

#### Admin dashboard

| Dashboard Desktop                 | Dashboard Mobile                |
| --------------------------------- | ------------------------------- |
| ![desktop](/docs/img/desktop.png) | ![mobile](/docs/img/mobile.png) |

#### Client pages variants

| Dark Mode                                       | Light Mode                                             |
| ----------------------------------------------- | ------------------------------------------------------ |
| ![dark mode countdown](/docs/img/countdown.png) | ![light mode countdown](/docs/img/countdown_light.png) |
