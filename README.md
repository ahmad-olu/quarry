# QUARRY

A scriptable, code-like API client for REST, ~~GraphQL,~~ ~~WebSockets,~~ and more.

quarry is a developer-first API client that blends the simplicity of *VSCode’s REST Client* with the power of *Postman* / *Bruno*. Write your requests in a declarative, code-like DSL with variables, comments, imports, and even logic-like string concatenation. Send, debug, and preview responses directly in your editor. Quarry is designed to grow with your stack — supporting REST, ~~GraphQL,~~ ~~gRPC,~~ ~~WebSockets,~~ ~~API simulations,~~ and beyond.

## syntax and structure

```qa
// Variables
let token = "abcd1234"
let base = "https://api.example.com"

// Request
name: getUsers
get ${base}/users
Authorization: Bearer ${token}

json {
  "limit": 20,
  "offset": 0
}

```

## Todo

mvp:

- Parser, interpreter for REST methods, `let`, `imports`, `basic interpolation`, `JSON/form/raw bodies`, `headers`.
- Save body, extract fields, `assert` comparison operators.
- CLI `flux run file`
