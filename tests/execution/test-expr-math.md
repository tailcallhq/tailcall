# expr logic

#### server:

```graphql
schema {
  query: Query
}

type Query {
  add: Int @expr(body: {add: [{const: 40}, {const: 2}]})
  subtract: Int @expr(body: {subtract: [{const: 52}, {const: 10}]})
  multiply: Float @expr(body: {multiply: [{const: 7}, {const: 6}]})
  mod: Int @expr(body: {mod: [{const: 1379}, {const: 1337}]})
  div1: Int @expr(body: {divide: [{const: 9828}, {const: 234}]})
  div2: Int @expr(body: {divide: [{const: 105}, {const: 2.5}]})
  inc: Int @expr(body: {inc: {const: 41}})
  dec: Int @expr(body: {dec: {const: 43}})
  product: Int @expr(body: {product: [{const: 7}, {const: 3}, {const: 2}]})
  sum: Int @expr(body: {sum: [{const: 20}, {const: 15}, {const: 7}]})
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { add subtract multiply mod div1 div2 inc dec product sum }
```
