# Grpc datasource

#### server:

```graphql
schema
  @server(port: 8000, graphiql: true)
  @upstream(httpCache: true, batch: {delay: 10})
  @link(id: "news.proto", src: "http://localhost:50051", type: ReflectionWithFileName) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:50051")
  newsById(news: NewsInput!): News!
    @grpc(method: "news.NewsService.GetNews", baseURL: "http://localhost:50051", body: "{{args.news}}")
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

#### mock:

```yml
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
    body: null

  response:
    status: 200
    body: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
