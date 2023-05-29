
## All routes

```sh
GET     /            # web UI
GET     /share/:slug # share UI
GET     /s/:slug     # short url redirecting to the target
GET     /store       # store all current shortcuts in a csv format file
GET     /backup      # Trigger backup for all shortcuts of the current instance to ./backups/<uuid>
PUT     /s           # put route for new entries during runtime
DELETE  /s           # deletes entries during runtime by slug
GET     /health      # readiness and liveness health
```

You can scale this url shortener to your requirements via seeding, since its hosting the same url's on all instances.
Seeding on launch from a `./seed.csv` is seen in the example section.

Notice the `/store` route that allows you to store the current shortcuts for your version in the csv format. This could be used to create a updated `./seed.csv` from one instance and redeploy all instances. This also allows "fresh" redeploys if you are mounting the seed.csv via a ConfigMap in a Kubernetes environment.

Also Zorka supports backups. On demand and on shutdown the database is dumped into a csv file for backups. You can trigger a backup via the /backup route.
If backups are present, Zorka will restore the database based on the backups and not on the initial seeding file.

The backups are stored based on a custom algorithm and custom .zorka files. More on .zorka files in [docs/en/zorka_files_and_backups.md](/docs/en/zorka_files_and_backups.md).

### What data is collected?

Zorka does store as little data as possible, since almost no data is needed to operate.

On one side you can safely use Zorka without leaking any of your private information to other companies. If you decide to protect the admin routes with OAuth2, the stored data for you is only the access token as a cookie.

For client pages no data is stored from the client or any data to the clients browser. This means customers wont be greeted with any annoying cookie or privacy banners.

## Protection

### Gated routes

These client pages support light and dark mode.

| Countdown                             | Approval                              | Blocker                               |
| ------------------------------------- | ------------------------------------- | ------------------------------------- |
| ![countdown](/docs/img/countdown.png) | ![approval](/docs/img/approval.png)   | ![blocker](/docs/img/blocker.png)     |

Countdown - behaives like a waiting lobby for not yet ready targets. Will reload the page after the countdown hits 0.
Approval - the URL is considered as 'untrusted' and the user should confirm the redirect.
Blocker - the redirect timed out and the redirect target is not reachable.

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

While you can have a custom skript generating this seed.csv file, you could also just configure redirects via the UI and download it straight from Zorka in the correct syntax.

### Configuration via configuration.yaml

>Option 3: You get most of the options from the well-known page a of OAuth2 service. For e.g. Google: https://accounts.google.com/.well-known/openid-configuration

>Also there is no "stay logged in" functionality. Refresh tokens are ignored and access tokens are cleaned up on browser close. Either the introspection verifies you or the access to admin pages is blocked. This should be good enough for backoffice access.

```yaml
auth: # Pick one option
  # Option 1: No Authentication
  none
  # Option 2a: Basic Authentication
  basic:
    username: username
    password: password
  # Option 2b: Basic Authentication exact (possible but just go with 2a)
  basic:
    header: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
  # Option 3: SSO via OAuth2 Code Flow
  oauth2:
    client_id: d675f5571ab33e44bb32
    client_secret: fb27917c2d78f59ef49f38e6454dad675f55718s
    scope: user
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
    button: continue
server:
  public_origin: http://localhost:8080
```

### Deployment via compose.yaml
```yaml
services:
  zorka:
    image: litvinav/zorka:latest
    ports:
    - 8080:8080
    volumes:
    - ./seed.csv:/app/seed.csv:ro
    - ./configuration.yaml:/app/configuration.yaml
    deploy:
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      resources:
        limits:
          cpus: '0.2'
          memory: 30M
        reservations:
          cpus: '0.1'
          memory: 20M
    cap_drop: [ALL]
```
If you are feeling paranoid or cannot use a amd64 image, you can always build Zorka from source and store the image in your registry.

### Preview

#### Internal pages

| Dashboard Desktop                 | Dashboard Mobile                | Sharing                           |
| --------------------------------- | ------------------------------- | --------------------------------- |
| ![desktop](/docs/img/desktop.png) | ![mobile](/docs/img/mobile.png) | ![sharing](/docs/img/sharing.png) |

#### Client pages variants

| Dark Mode                                       | Light Mode                                             |
| ----------------------------------------------- | ------------------------------------------------------ |
| ![dark mode countdown](/docs/img/countdown.png) | ![light mode countdown](/docs/img/countdown_light.png) |
