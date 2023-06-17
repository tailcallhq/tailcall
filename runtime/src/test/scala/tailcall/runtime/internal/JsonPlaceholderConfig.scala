package tailcall.runtime.internal

import tailcall.runtime.JsonT
import tailcall.runtime.http.Method
import tailcall.runtime.model.Config.{Arg, Field, Type}
import tailcall.runtime.model.Operation
import tailcall.runtime.model.{Config, Path}

import java.net.URI

object JsonPlaceholderConfig {
  private def createUser = users.withMethod(Method.POST).withBody(Option("{{args.user}}"))
  private def posts      = Operation.Http(Path.unsafe.fromString("/posts"))
  private def postsById  = Operation.Http(Path.unsafe.fromString("/posts/{{args.id}}"))
  private def userById   = Operation.Http(Path.unsafe.fromString("/users/{{userId}}"))
  private def userPosts  = Operation.Http(Path.unsafe.fromString("/users/{{value.id}}/posts"))
  private def users      = Operation.Http(Path.unsafe.fromString("/users"))

  val config: Config = Config.default.withMutation("Mutation")
    .withBaseURL(URI.create("https://jsonplaceholder.typicode.com").toURL).withTypes(
      "Mutation"   -> Type(
        "createUser" -> Field.ofType("Id").withSteps(createUser)
          .withArguments(Map("user" -> Arg.ofType("NewUser").asRequired.withDoc("User as an argument.")))
      ),
      "Id"         -> Type.empty.withDoc("An Id container.").withFields("id" -> Field.int.asRequired),
      "Query"      -> Type(
        "posts" -> Field.ofType("Post").withSteps(posts).asList.withDoc("A list of all posts."),
        "users" -> Field.ofType("User").withSteps(users).asList.withDoc("A list of all users."),
        "post" -> Field.ofType("Post").withSteps(postsById)("id" -> Arg.int.asRequired).withDoc("A single post by id."),
        "user" -> Config.Field("User", Operation.transform(JsonT.objPath("userId" -> List("args", "id"))), userById)(
          "id" -> Arg.int.asRequired
        ).withDoc("A single user by id."),
        "unusedField" -> Field.str.withOmit(true).withDoc("An unused field that will be omitted."),
      ),
      "NewUser"    -> Type.empty.withDoc("A new user.").withFields(
        "name"     -> Field.str.asRequired,
        "username" -> Field.str.asRequired,
        "email"    -> Field.str.asRequired,
        "address"  -> Field.ofType("NewAddress"),
        "phone"    -> Field.str,
        "website"  -> Field.str,
        "company"  -> Field.ofType("NewCompany"),
      ),
      "User"       -> Type(
        "id"       -> Field.int.asRequired,
        "name"     -> Field.str.asRequired,
        "username" -> Field.str.asRequired,
        "email"    -> Field.str.asRequired,
        "address"  -> Field.ofType("Address"),
        "phone"    -> Field.str,
        "website"  -> Field.str,
        "company"  -> Field.ofType("Company"),
        "posts"    -> Field.ofType("Post").withSteps(userPosts).asList,
      ),
      "Post"       -> Type(
        "id"     -> Field.int.asRequired,
        "userId" -> Field.int.asRequired,
        "title"  -> Field.str,
        "body"   -> Field.str,
        "user"   -> Field.ofType("User")
          .withSteps(Operation.transform(JsonT.objPath("userId" -> (List("value", "userId")))), userById),
      ),
      "Address"    -> Type(
        "street"  -> Field.str,
        "suite"   -> Field.str,
        "city"    -> Field.str,
        "zipcode" -> Field.str.withName("zip"),
        "geo"     -> Field.ofType("Geo"),
      ),
      "NewAddress" -> Type(
        "street"  -> Field.str,
        "suite"   -> Field.str,
        "city"    -> Field.str,
        "zipcode" -> Field.str.withName("zip"),
        "geo"     -> Field.ofType("NewGeo"),
      ),
      "Company"    -> Type("name" -> Field.str, "catchPhrase" -> Field.str, "bs" -> Field.str),
      "NewCompany" -> Type("name" -> Field.str, "catchPhrase" -> Field.str, "bs" -> Field.str),
      "Geo"        -> Type("lat" -> Field.str, "lng" -> Field.str),
      "NewGeo"     -> Type("lat" -> Field.str, "lng" -> Field.str),
    )
}
