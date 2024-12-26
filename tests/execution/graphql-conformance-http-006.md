# Test complex nested query

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
  profilePic(size: Int, width: Int, height: Int): String!
    @expr(body: "{{.value.id}}_{{.args.size}}_{{.args.width}}_{{.args.height}}")
  friends(first: Int): [User!]!
    @http(
      url: "http://upstream/friends"
      query: [{key: "id", value: "{{.value.id}}"}, {key: "first", value: "{{.args.first}}"}]
    )
  mutualFriends(first: Int): [User!]!
    @http(
      url: "http://upstream/mutual-friends"
      query: [{key: "id", value: "{{.value.id}}"}, {key: "first", value: "{{.args.first}}"}]
    )
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 3
  response:
    status: 200
    body:
      id: 4
      name: User 4
- request:
    method: GET
    url: http://upstream/friends?id=4&first=10
  expectedHits: 3
  response:
    status: 200
    body:
      - id: 1
        name: friend_1
        profilePic: friend_1_pic
      - id: 2
        name: friend_2
        profilePic: friend_2_pic
      - id: 3
        name: friend_3
        profilePic: friend_3_pic
- request:
    method: GET
    url: http://upstream/mutual-friends?id=4&first=10
  expectedHits: 3
  response:
    status: 200
    body:
      - id: 1
        name: mutual_friend_1
        profilePic: mutual_friend_1_pic
      - id: 2
        name: mutual_friend_2
        profilePic: mutual_friend_2_pic
      - id: 3
        name: mutual_friend_3
        profilePic: mutual_friend_3_pic
```

```yml @test
# Positve: query
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            id
            name
            profilePic(size: 50)
          }
          mutualFriends(first: 10) {
            id
            name
            profilePic(size: 50)
          }
        }
      }
# Positve: fragment simple
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            ...friendFields
          }
          mutualFriends(first: 10) {
            ...friendFields
          }
        }
      }

      fragment friendFields on User {
        id
        name
        profilePic(size: 50)
      }
# Positve: fragment nested
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          friends(first: 10) {
            ...friendFields
          }
          mutualFriends(first: 10) {
            ...friendFields
          }
        }
      }

      fragment friendFields on User {
        id
        name
        ...standardProfilePic
      }

      fragment standardProfilePic on User {
        profilePic(size: 50)
      }

# Negative: missing fragment
# TODO: Disabled because async_graphql::dynamic does not perform validation
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       query {
#         user(id: 4) {
#           friends(first: 10) {
#             ...friendFields
#           }
#           mutualFriends(first: 10) {
#             ...friendFields
#           }
#         }
#       }

#       fragment friendFields on User {
#         id
#         name
#         ...standardProfilePic
#       }
```
