# Call multiple steps

```graphql @server
schema {
  query: Query
}

type Query {
  a(input: JSON): JSON @const(data: "{{args.input.a}}")
  b(input: JSON): JSON @const(data: "{{args.input.b}}")
  c(input: JSON): JSON @const(data: "{{args.input.c}}")

  abc(input: JSON): JSON
    @call(
      steps: [
        { query: "a", args: { input: "{{args.input}}" } }
        { query: "b", args: { input: "{{args.input}}" } }
        { query: "c", args: { input: "{{args.input}}" } }
      ]
    )
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'query { abc(input: { a: 1, b: 2, c: 3 }) }'
```