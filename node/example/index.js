const tc = require("@tailcallhq/wasm-node")

async function run() {
  await tc.main()
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
    const executor = new tc.GraphQLExecutor(schema, source)

    // Execute a query
    const query = "{ user(id: 2) { id } }"
    console.log(await executor.execute(query))
  } catch (error) {
    console.log("Error executing GraphQL query:" + error)
  }
}

run()
