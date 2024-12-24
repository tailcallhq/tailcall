# Optional input fields

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user(id: ID!): User! @http(url: "http://upstream/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  profilePic(size: Int! = 100, width: Int, height: Int = 100): String!
    @expr(body: "{{.value.id}}_{{.args.size}}_{{.args.width}}_{{.args.height}}")
  featuredVideo(video: VideoSize! = {width: 1600, height: 900}): String!
    @expr(body: "video_{{.value.id}}_{{.args.video.width}}_{{.args.video.height}}_{{.args.video.hdr}}")
  featuredVideoPreview(video: VideoSize! = {}): String!
    @expr(body: "video_{{.value.id}}_{{.args.video.width}}_{{.args.video.height}}_{{.args.video.hdr}}")
  searchComments(query: [[String!]!]! = [["today"]]): String! @expr(body: "video_{{.value.id}}_{{.args.query}}")
  spam(foo: [Foo!]!): String! @expr(body: "FIZZ: {{.args.foo}}")
}

input VideoSize {
  width: Int!
  height: Int!
  hdr: Boolean = true
}

input Foo {
  bar: String! = "BUZZ"
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 10
  response:
    status: 200
    body:
      id: 4
      name: User 4
```

```yml @test
# Positve: no optional
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic
        }
      }
# Positve: different size
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(size: 200)
        }
      }
# Positve: width only
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200)
        }
      }
# Positve: width only, unset height
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200, height: null)
        }
      }
# Positve: width and height
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(width: 200, height: 50)
        }
      }
# Positve: video default
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          featuredVideo
        }
      }
# Positve: video overwrite
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          featuredVideo(video: {width: 1920, height: 1080, hdr: true})
        }
      }
# Positve: comments default
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          searchComments
        }
      }
# Positve: comments overwrite
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          searchComments(query: [["test", "tost"], ["foo"], ["bar"], ["bizz", "buzz"]])
        }
      }

# # Positve: defaults from input
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          featuredVideoPreview
        }
      }

# Negative: invalid size
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          profilePic(size: null)
        }
      }

# Positve: array fields
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          spam(foo: [{}, { bar: "test"}])
        }
      }
```
