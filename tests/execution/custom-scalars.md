# Using all custom scalars

```graphql @schema
schema {
  query: Query
}
schema {
  query: Query
}

schema {
  query: Query
}

type Query {
  qDate: Date @expr(body: "2023-10-05T00:00:00Z")
  qDateTime: DateTime @expr(body: "2023-10-05T00:00:00Z")
  qEmail: Email @expr(body: "funny@not.com")
  qUrl: Url @expr(body: "http://example.com")
  qPhoneNumber: PhoneNumber @expr(body: "+1234567890")
  qJSON: JSON @expr(body: {a: 1, b: 2})
  qBytes: Bytes @expr(body: "1")
  qUInt128: UInt128 @expr(body: "2")
  qUInt16: UInt16 @expr(body: 3)
  qUInt32: UInt32 @expr(body: 4)
  qUInt64: UInt64 @expr(body: "5")
  qUInt8: UInt8 @expr(body: 6)
  qInt128: Int128 @expr(body: "7")
  qInt16: Int16 @expr(body: 8)
  qInt32: Int32 @expr(body: 9)
  qInt64: Int64 @expr(body: "10")
  qInt8: Int8 @expr(body: 11)
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      {
          qBytes
          qDate
          qDateTime
          qEmail
          qInt128
          qInt16
          qInt32
          qInt64
          qInt8
          qJSON
          qPhoneNumber
          qUInt128
          qUInt16
          qUInt32
          qUInt64
          qUInt8
          qUrl
      }
```
