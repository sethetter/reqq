# reqq

A cli for making HTTP requests from predefined request files.

Expects available requests to be in a local `.reqq` folder.

Environments can be configured with arbitrary variables that can be embedded in request
files inside of an `.reqq/envs/` folder.

## Install

```
cargo install reqq
```

## Usage

Request file at `.reqq/create-user.reqq`.

```
POST {{ baseUrl }}/api/v1/users
X-Secret-Header: {{ secret }}
{ "username": "yep", "password": "nope" }
```

Env config at `.reqq/envs/test.json`.

```
{ "baseUrl": "https://example.com", "secret": "lolol" }
```

Then this command will issue the request!

```
reqq --env=test create-user
```

If you provide no environment, it will attempt to load `.reqq/envs/default.json`.

## `.reqq` files

Reqq uses [handlebars](https://docs.rs/handlebars/3.4.0/handlebars/) as the templating
engine, so anything that's fair game there is fair game in `.reqq` files.

## Commands

- `reqq [--env=<env>] <request>`, executes a request.
- `reqq list`, lists all available requests.
- `reqq envs`, lists available envs.
