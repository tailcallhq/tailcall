# Grpc datasource

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
      method: "GetNews"
      baseURL: "http://localhost:50051"
      body: "{{args.news}}"
      protoPath: "src/grpc/tests/news.proto"
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
      url: http://localhost:50051/NewsService/GetAllNews
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { news {news{ id }} }
env: {}
```
