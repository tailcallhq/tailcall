# Test scalars related to integer representation

```graphql @config
schema @server(port: 8000, hostname: "localhost") {
  query: Query
}

type Query {
  int64(x: Int64): Int64 @expr(body: "{{.args.x}}")
  int64Str(x: Int64Str): Int64Str @expr(body: "{{.args.x}}")
  uInt32(x: UInt32): UInt32 @expr(body: "{{.args.x}}")
  uInt64(x: UInt64): UInt64 @expr(body: "{{.args.x}}")
  uInt64Str(x: UInt64Str): UInt64Str @expr(body: "{{.args.x}}")
}
```

```yml @test
# TODO: some of the tests do not work properly due to the issue https://github.com/tailcallhq/tailcall/issues/2039

# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: int64(x: -47486461565) b: int64(x: 32147483647) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int64Str(x: "-46486461565") b: int64Str(x: "32147483647") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: uInt32(x: 125) b: uInt32(x: 3147483647) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: uInt64(x: 47486461565) b: uInt64(x: 9823372036854775807) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uInt64Str(x: "47486461565") b: uInt64Str(x: "9823372036854775807") }'

# Invalid value test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int64(x: 32147483647) b: int64(x: "48645648645") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int64Str(x: -45486461565) b: int64Str(x: "3214448646163165487483647") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uInt32(x: -125) b: uInt32(x: "3147483647") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uInt64(x: "47486461565") b: uInt64(x: -156) }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uInt64Str(x: 47486461565) b: uInt64Str(x: "19823372036854775807") }'
```
