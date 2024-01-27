# Grpc datasource with batching

#### server:
```graphql
schema @server(port: 8000, graphiql: true) @upstream(httpCache: true, batch: {delay: 10}) {
  query: Query
}

type Query {
  news: NewsData!
    @grpc(
      service: "NewsService"
      method: "GetAllNews"
      baseURL: "http://localhost:50051"
      protoPath: "src/grpc/tests/news.proto"
    )
  newsById(news: NewsInput!): News!
    @grpc(
      service: "NewsService"
      method: "GetMultipleNews"
      baseURL: "http://localhost:50051"
      body: "{{args.news}}"
      protoPath: "src/grpc/tests/news.proto"
      groupBy: ["news", "id"]
    )
}
input NewsInput {
  id: Int
  title: String
  body: String
  postImage: String
}
type NewsData {
  news: [News]!
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}
```

#### assert:
```yml
mock:
- request:
    method: POST
    url: http://localhost:50051/NewsService/GetMultipleNews
    headers: {}
    body: \0\0\0\0\n\x02\x08\x02\n\x02\x08\x03
  response:
    status: 200
    headers: {}
    body: \0\0\0\0t\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2\n#\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3
- request:
    method: POST
    url: http://localhost:50051/NewsService/GetMultipleNews
    headers: {}
    body: \0\0\0\0\n\x02\x08\x03\n\x02\x08\x02
  response:
    status: 200
    headers: {}
    body: \0\0\0\0t\n#\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { newsById2: newsById(news: {id: 2}) { title }, newsById3: newsById(news: {id: 3}) { title } }'
env: {}
```
