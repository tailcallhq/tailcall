# test-scalar-email

#### server:

```graphql
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

#### assert:

```yml
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
```
