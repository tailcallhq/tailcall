# test-scalar-email

```graphql @server
schema @server(hostname: "localhost", port: 8000) {
  query: Query
}

scalar AnyScalar

scalar Email

type Query {
  any(value: AnyScalar!): AnyScalar @expr(body: "{{.args.value}}")
  date(value: Date!): Date! @expr(body: "{{.args.value}}")
  email(value: Email!): Email! @expr(body: "{{.args.value}}")
  phone(value: PhoneNumber!): PhoneNumber! @expr(body: "{{.args.value}}")
  url(value: Url!): Url! @expr(body: "{{.args.value}}")
}
```

```yml @test
# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ email(value: "hello@valid.com") }'
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ phone(value: "+1 (614) 1234567") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ date(value: "2023-03-08T12:45:26-05:00") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ url(value: "https://tailcall.run/") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ any1: any(value: { test: "abc" } ), any2: any(value: "string-value") }'

# Invalid value test

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ email(value: "hello@invalid") }'
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ phone(value: "0") }'
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ phone(value: "1234567890") }'
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ date(value: "2023-03-08T12:45") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ url(value: "invalid_host") }'
```
