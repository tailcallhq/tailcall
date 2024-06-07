# Test scalars related to integer representation

```graphql @config
schema @server(port: 8000, hostname: "localhost") {
  query: Query
}

type Query {
  int8(x: Int8): Int8 @expr(body: "{{.args.x}}")
  int16(x: Int16): Int16 @expr(body: "{{.args.x}}")
  int32(x: Int32): Int32 @expr(body: "{{.args.x}}")
  int64(x: Int64): Int64 @expr(body: "{{.args.x}}")
  int128(x: Int128): Int128 @expr(body: "{{.args.x}}")

  uint8(x: UInt8): UInt8 @expr(body: "{{.args.x}}")
  uint16(x: UInt16): UInt16 @expr(body: "{{.args.x}}")
  uint32(x: UInt32): UInt32 @expr(body: "{{.args.x}}")
  uint64(x: UInt64): UInt64 @expr(body: "{{.args.x}}")
  uint128(x: UInt128): UInt128 @expr(body: "{{.args.x}}")
}
```

```yml @test
# TODO: some of the tests do not work properly due to the issue https://github.com/tailcallhq/tailcall/issues/2039

# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: int8(x: -125) b: int8(x: 120) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: int16(x: -1250) b: int16(x: 32767) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: int32(x: -125) b: int32(x: 2147483645) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int64(x: "-128") b: int64(x: "9223372036854775") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int128(x: "-125") b: int128(x: "1701411834604692317316873037158841057") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: uint8(x: 0) b: uint8(x: 255) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: uint16(x: 32767) b: uint16(x: 63535) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: uint32(x: 65535) b: uint32(x: 4294967295) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint64(x: "0") b: uint64(x: "18446744073709551615") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int128(x: "1564") b: int128(x: "340282366920938463463374607431768211455") }'

# Invalid value test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int8(x: 3214) b: int8(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int16(x: -42768) b: int16(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int32(x: 4147483647) b: int32(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int64(x: 3214) b: int64(x: "92233720368547758090") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int128(x: 3214) b: int128(x: "3170141183460469231731687303715884105727") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint8(x: 280) b: uint8(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint16(x: -1) b: uint16(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint32(x: 8294967295) b: uint32(x: "48") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint64(x: "-1") b: uint64(x: "3518446744073709551615") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: uint128(x: -1) b: uint128(x: "640282366920938463463374607431768211455") }'
```
