package tailcall.runtime.internal

import tailcall.runtime.ast.Path
import tailcall.runtime.dsl.Config
import tailcall.runtime.dsl.Config.{Arg, Field, Step, Type}
import tailcall.runtime.http.Method

object JsonPlaceholderConfig {
  def createUser: Step.Http = users.withMethod(Method.POST)
  def posts                 = Config.Step.Http(Path.unsafe.fromString("/posts"))
  def postsById             = Config.Step.Http(Path.unsafe.fromString("/posts/{{args.id}}"))
  def userById              = Config.Step.Http(Path.unsafe.fromString("/users/{{userId}}"))
  def userPosts: Step       = Config.Step.Http(Path.unsafe.fromString("/users/{{value.id}}/posts"))
  def users                 = Config.Step.Http(Path.unsafe.fromString("/users"))

  val graphQL = Config.GraphQL(
    schema = Config.RootSchema(query = Some("Query"), mutation = Some("Mutation")),
    types = Map(
      "Mutation"   -> Type(
        "createUser" -> Field.ofType("Id").withSteps(createUser)
          .withArguments(Map("user" -> Arg.ofType("NewUser").asRequired.withDoc("The user to create")))
      ),
      "Id"         -> Type.empty
        .withDoc("A general purpose Id container, used when an object is created an only the Id is returned")
        .withFields("id" -> Field.int.asRequired),
      "Query"      -> Type(
        "posts" -> Field.ofType("Post").withSteps(posts).asList.withDoc("A list of all posts"),
        "users" -> Field.ofType("User").withSteps(users).asList.withDoc("A list of all users"),
        "post"  -> Field.ofType("Post").withSteps(postsById)("id" -> Arg.int.asRequired).withDoc("A single post"),
        "user"  -> Config
          .Field("User", Config.Step.ObjPath("userId" -> List("args", "id")), userById)("id" -> Arg.int.asRequired)
          .withDoc("A single user"),
      ),
      "NewUser"    -> Type(
        "name"     -> Field.string.asRequired,
        "username" -> Field.string.asRequired,
        "email"    -> Field.string.asRequired,
        "address"  -> Field.ofType("NewAddress"),
        "phone"    -> Field.string,
        "website"  -> Field.string,
        "company"  -> Field.ofType("NewCompany"),
      ),
      "User"       -> Type(
        "id"       -> Field.int.asRequired,
        "name"     -> Field.string.asRequired,
        "username" -> Field.string.asRequired,
        "email"    -> Field.string.asRequired,
        "address"  -> Field.ofType("Address"),
        "phone"    -> Field.string,
        "website"  -> Field.string,
        "company"  -> Field.ofType("Company"),
        "posts"    -> Field.ofType("Post").withSteps(userPosts).asList,
      ),
      "Post"       -> Type(
        "id"     -> Field.int.asRequired,
        "userId" -> Field.int.asRequired,
        "title"  -> Field.string,
        "body"   -> Field.string,
        "user"   -> Field.ofType("User").withSteps(Config.Step.ObjPath("userId" -> List("value", "userId")), userById),
      ),
      "Address"    -> Type(
        "street"  -> Field.string,
        "suite"   -> Field.string,
        "city"    -> Field.string,
        "zipcode" -> Field.string,
        "geo"     -> Field.ofType("Geo"),
      ),
      "NewAddress" -> Type(
        "street"  -> Field.string,
        "suite"   -> Field.string,
        "city"    -> Field.string,
        "zipcode" -> Field.string,
        "geo"     -> Field.ofType("NewGeo"),
      ),
      "Company"    -> Type("name" -> Field.string, "catchPhrase" -> Field.string, "bs" -> Field.string),
      "NewCompany" -> Type("name" -> Field.string, "catchPhrase" -> Field.string, "bs" -> Field.string),
      "Geo"        -> Type("lat" -> Field.string, "lng" -> Field.string),
      "NewGeo"     -> Type("lat" -> Field.string, "lng" -> Field.string),
    ),
  )

  val server = Config.Server(baseURL = Option(new java.net.URL("https://jsonplaceholder.typicode.com")))
  val config = Config(server = server, graphQL = graphQL)
}
