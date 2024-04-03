# Call multiple steps piping

```graphql @server
schema {
  query: Query
}

type Query {
  a(input: JSON): JSON @const(data: "{{args.input.a}}")
  a_input(input: JSON): JSON @const(data: {input: "{{args.input.a}}"})
  abc(input: JSON): JSON @call(steps: [{query: "a", args: {input: "{{args.input}}"}}, {query: "wrap_args"}, {query: "b"}, {query: "wrap_args"}, {query: "c"}])
  abc_input(input: JSON): JSON @call(steps: [{query: "wrap_input", args: {input: "{{args.input}}"}}, {query: "a_input"}, {query: "wrap_input"}, {query: "b_input"}, {query: "wrap_input"}, {query: "c"}])
  b(input: JSON): JSON @const(data: "{{args.input.b}}")
  b_input(input: JSON): JSON @const(data: {input: "{{args.input.b}}"})
  c(input: JSON): JSON @const(data: "{{args.input.c}}")
  wrap_args: JSON @const(data: {input: "{{args}}"})
  wrap_input(input: JSON): JSON @const(data: {input: "{{args.input}}"})
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc_input(input: {a: {b: {c: 3}}})}"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc(input: {a: {b: {c: 3}}}) }"
```
