# Call multiple steps piping


```graphql @server
schema {
  query: Query
}

type Query {
  a(input: JSON): JSON @const(data: "{{args.input.a}}")
  b: JSON @const(data: "{{args.b}}")
  c: JSON @const(data: "{{args.c}}")

  abc(input: JSON): JSON
    @call(
      steps: [
        {query: "a", args: {input: "{{args.input}}"}}
        {query: "b"}
        {query: "c"}
      ]
    )
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc(input: { a: { b: { c: 3 } }})}"
```
