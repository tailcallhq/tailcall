# test-scalar-email

```graphql @server
scalar Email
scalar PhoneNumber
scalar Date
scalar Url

schema @server(port: 8000, graphiql: true, hostname: "localhost") {
  query: Query
}

type Query {
  email(value: Email!): Email! @const(data: "{{args.value}}")
  phone(value: PhoneNumber!): PhoneNumber! @const(data: "{{args.value}}")
  date(value: Date!): Date! @const(data: "{{args.value}}")
  url(value: Url!): Url! @const(data: "{{args.value}}")
}
```

```yml @assert
# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ email(value: "alo@valid.com") }'
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

# Invalid value test

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ email(value: "alo@invalid") }'
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
    query: '{ url(value: "invalidhost") }'
```
