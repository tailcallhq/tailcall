package tailcall.gateway.internal

import tailcall.gateway.ast.{Path, TSchema}
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config.Step

object JsonPlaceholderConfig {

  val Address: TSchema = TSchema.obj(
    "street"  -> TSchema.string,
    "suite"   -> TSchema.string,
    "city"    -> TSchema.string,
    "zipcode" -> TSchema.string,
    "geo"     -> TSchema.obj("lat" -> TSchema.string, "lng" -> TSchema.string)
  )

  val Company: TSchema = TSchema.obj("name" -> TSchema.string, "catchPhrase" -> TSchema.string, "bs" -> TSchema.string)

  val User: TSchema = TSchema.obj(
    "id"       -> TSchema.int,
    "name"     -> TSchema.string,
    "username" -> TSchema.string,
    "email"    -> TSchema.string,
    "address"  -> Address,
    "company"  -> Company
  )

  val Post = TSchema
    .obj("id" -> TSchema.int, "userId" -> TSchema.int, "title" -> TSchema.string, "body" -> TSchema.string)

  val users           = Config.Step.Http(Path.unsafe.fromString("/users")).withOutput(TSchema.arr(User))
  val userById        = Config.Step.Http(Path.unsafe.fromString("/users/{{args.id}}")).withOutput(User)
  val postsById       = Config.Step.Http(Path.unsafe.fromString("/posts/{{args.id}}")).withOutput(Post)
  val posts           = Config.Step.Http(Path.unsafe.fromString("/posts")).withOutput(TSchema.arr(Post))
  val userPosts: Step = Config.Step.Http(Path.unsafe.fromString("/users/{{value.id}}/posts"))
    .withOutput(TSchema.arr(Post))
  val postUser: Step  = Config.Step.Http(Path.unsafe.fromString("/users/{{value.userId}}")).withOutput(User)

  val graphQL = Config.GraphQL(
    schema = Config.SchemaDefinition(query = Some("Query"), mutation = Some("Mutation")),
    types = Map(
      "Query"   -> Map(
        "posts" -> Config.Field("Post", posts).asList,
        "users" -> Config.Field("User", users).asList
        // "post"  -> Config.Field("Post", postsById)("id" -> Argument.int.asRequired),
        // "user"  -> Config.Field("User", userById)("id" -> Argument.int.asRequired)
      ),
      "User"    -> Map(
        "id"       -> Config.Field.int.asRequired,
        "name"     -> Config.Field.string.asRequired,
        "username" -> Config.Field.string.asRequired,
        "email"    -> Config.Field.string.asRequired,
        "address"  -> Config.Field("Address"),
        "phone"    -> Config.Field.string,
        "website"  -> Config.Field.string,
        "company"  -> Config.Field("Company"),
        "posts"    -> Config.Field("Post", userPosts).asList
      ),
      "Post"    -> Map(
        "id"     -> Config.Field.int.asRequired,
        "userId" -> Config.Field.int.asRequired,
        "title"  -> Config.Field.string,
        "body"   -> Config.Field.string,
        "user"   -> Config.Field("User", postUser)
      ),
      "Address" -> Map(
        "street"  -> Config.Field.string,
        "suite"   -> Config.Field.string,
        "city"    -> Config.Field.string,
        "zipcode" -> Config.Field.string,
        "geo"     -> Config.Field("Geo")
      ),
      "Company" -> Map(
        "name"        -> Config.Field.string,
        "catchPhrase" -> Config.Field.string,
        "bs"          -> Config.Field.string
      ),
      "Geo"     -> Map("lat" -> Config.Field.string, "lng" -> Config.Field.string)
    )
  )

  val server = Config.Server(host = "jsonplaceholder.typicode.com", port = Option(443))
  val config = Config(server = server, graphQL = graphQL)
}
