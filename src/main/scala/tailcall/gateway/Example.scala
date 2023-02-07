package tailcall.gateway

import tailcall.gateway.ast._
import zio.schema.Schema

object Example extends App {
  val User = TSchema.structured(
    "id"    -> TSchema.int,
    "name"  -> TSchema.string,
    "phone" -> TSchema.string,
    "email" -> TSchema.string
  )

  val Post = TSchema
    .structured("id" -> TSchema.int, "title" -> TSchema.string, "body" -> TSchema.string)

  val typicode = Endpoint.inet("jsonplaceholder.typicode.com")

  val usersEndpoint: Endpoint = Endpoint
    .http(path = Path.unsafe.fromString("/users"), address = typicode, output = TSchema.arr(User))

  val postsEndpoint: Endpoint = Endpoint
    .http(path = Path.unsafe.fromString("/posts"), address = typicode, output = TSchema.arr(Post))

  val userPostsEndpoint = Endpoint.http(
    path = Path.unsafe.fromString("/users/{userId}/posts"),
    address = typicode,
    input = TSchema.structured("userId" -> TSchema.int),
    output = TSchema.arr(Post)
  )

  val manyUserPostsEndpoint = Endpoint.http(
    path = Path.unsafe.fromString("/posts"),
    query = Map("userId" -> "userId"),
    address = typicode,
    input = TSchema.structured("userId" -> TSchema.arr(TSchema.int)),
    output = TSchema.arr(Post)
  )

  val orch = Orc.obj(
    "Query" -> Orc
      .obj("users" -> Orc.endpoint(usersEndpoint), "posts" -> Orc.endpoint(postsEndpoint)),
    "User"  -> Orc.obj(
      "posts" -> Orc.spec("userId" -> Orc.context("id")).pipe(Orc.endpoint(userPostsEndpoint)),
      "fullName" -> (Orc.context("firstName") ++ Orc.context("lastName"))
    ),
    "Post"  -> Orc.obj(
      "user" -> Orc.batch(
        endpoint = Orc.spec("userId" -> Orc.context("userId"))
          .pipe(Orc.endpoint(manyUserPostsEndpoint)),
        groupBy = Orc.path("userId")
      )
    )
  )

  pprint.pprintln(Schema[Orc].toDynamic(orch))
}
