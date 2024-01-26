import {GraphQLExecutor} from "tc-wasm"
async function run() {
  try {
    const schema =
      "schema\n" +
      '  @server(port: 8000, graphiql: true, hostname: "0.0.0.0")\n' +
      '  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) {\n' +
      "  query: Query\n" +
      "}\n" +
      "\n" +
      "type Query {\n" +
      '  posts: [Post] @http(path: "/posts")\n' +
      '  users: [User] @http(path: "/users")\n' +
      '  user(id: Int!): User @http(path: "/users/{{args.id}}")\n' +
      "}\n" +
      "\n" +
      "type User {\n" +
      "  id: Int!\n" +
      "  name: String!\n" +
      "  username: String!\n" +
      "  email: String!\n" +
      "  phone: String\n" +
      "  website: String\n" +
      "}\n" +
      "\n" +
      "type Post {\n" +
      "  id: Int!\n" +
      "  userId: Int!\n" +
      "  title: String!\n" +
      "  body: String!\n" +
      '  user: User @http(path: "/users/{{value.userId}}")\n' +
      "}\n"
    const source = ".graphql"
    const executor = new GraphQLExecutor(schema, source)

    // Execute a query
    const query = "{ user(id: 2) { id } }"
    return JSON.parse(await executor.execute(query))
  } catch (error) {
    return {err: '"Error executing GraphQL query:" + error'}
  }
}

let res = await run()
document.getElementById("content").textContent = JSON.stringify(res, null, 2)
