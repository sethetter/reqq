# reqq

A cli for making HTTP requests from predefined request files.

Expects available requests to be in a local `.reqq` folder.

Environments can be configured with arbitrary variables that can be embedded in request
files inside of an `.reqq/envs/` folder.

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

## Commands

- `reqq [--env=<env>] <request>`, executes a request.
- `reqq list`, lists all available requests.
- `reqq envs`, lists available envs.
