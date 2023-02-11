package tailcall.gateway

import tailcall.gateway.ast.{Endpoint, Orc, TSchema}
import tailcall.gateway.remote.Remote
import zio.schema.DynamicValue
import zio.test._

object OrcSpec extends ZIOSpecDefault {

  object schema {
    val User: TSchema = TSchema.obj("id" -> TSchema.int, "name" -> TSchema.str)

    val Post: TSchema    = TSchema.obj(
      "id"     -> TSchema.int,
      "title"  -> TSchema.str,
      "body"   -> TSchema.str,
      "userId" -> TSchema.int
    )
    val Comment: TSchema = TSchema.obj(
      "id"     -> TSchema.int,
      "name"   -> TSchema.str,
      "email"  -> TSchema.str,
      "body"   -> TSchema.str,
      "postId" -> TSchema.int
    )

    val Album: TSchema = TSchema
      .obj("id" -> TSchema.int, "title" -> TSchema.str, "userId" -> TSchema.int)

    val Photo: TSchema = TSchema.obj(
      "id"           -> TSchema.int,
      "title"        -> TSchema.str,
      "url"          -> TSchema.str,
      "thumbnailUrl" -> TSchema.str,
      "albumId"      -> TSchema.int
    )

    val Todo: TSchema = TSchema.obj(
      "id"        -> TSchema.int,
      "title"     -> TSchema.str,
      "completed" -> TSchema.bool,
      "userId"    -> TSchema.int
    )
  }

  object endpoints {
    val typicode: Endpoint  = Endpoint.make("jsonplaceholder.typicode.com")
    val users: Endpoint     = typicode.withPath("/users").withOutput(TSchema.arr(schema.User))
    val posts: Endpoint     = typicode.withPath("/posts").withOutput(TSchema.arr(schema.Post))
    val userPosts: Endpoint = typicode
      .withPath("/posts")
      .withQuery("userId" -> "${userId}")
      .withInput(TSchema.obj("userId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Post))

    val postComments: Endpoint = typicode
      .withPath("/comments")
      .withQuery("postId" -> "${postId}")
      .withInput(TSchema.obj("postId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Comment))

    val userComments: Endpoint = typicode
      .withPath("/comments")
      .withQuery("email" -> "${email}")
      .withInput(TSchema.obj("email" -> TSchema.str))
      .withOutput(TSchema.arr(schema.Comment))

    val userAlbums: Endpoint = typicode
      .withPath("/albums")
      .withQuery("userId" -> "${userId}")
      .withInput(TSchema.obj("userId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Album))

    val UserTodos: Endpoint = typicode
      .withPath("/todos")
      .withQuery("userId" -> "${userId}")
      .withInput(TSchema.obj("userId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Todo))

    val AlbumPhotos: Endpoint = typicode
      .withPath("/photos")
      .withQuery("albumId" -> "${albumId}")
      .withInput(TSchema.obj("albumId" -> TSchema.int))
      .withOutput(TSchema.arr(schema.Photo))

    val ManyUsersPosts: Endpoint = typicode
      .withPath("/posts")
      .withQuery("userId" -> "${userId}")
      .withInput(TSchema.obj("userId" -> TSchema.arr(TSchema.int)))
      .withOutput(TSchema.arr(schema.Post))
  }

  val unit: Remote[DynamicValue] = Remote.dynamicValue(())

  val query: Orc = Orc.query(
    "Query" -> List(
      "users" -> (_ => endpoints.users(unit)),
      "posts" -> (_ => endpoints.posts(unit))
    ),
    "User"  -> List(
      "posts" -> { context => endpoints.userPosts(Remote.record("userId" -> context.value)) },
      "manyUsersPosts" -> { context =>
        Remote.batch(
          endpoints.ManyUsersPosts(Remote.record("userId" -> context.value.path("id").getOrDie)),
          List("userId")
        )
      },
      "fullName"       -> { context =>
        val fn = context.value.path("firstName").flatMap(_.asString).getOrDie
        val ln = context.value.path("lastName").flatMap(_.asString).getOrDie
        Remote.dynamicValue(fn ++ Remote(" ") ++ ln)
      },
      "comments"       -> { context =>
        endpoints.userComments(Remote.record("email" -> context.value.path("email").getOrDie))
      },
      "albums"         -> { context =>
        endpoints.userAlbums(Remote.record("userId" -> context.value.path("id").getOrDie))
      },
      "todos"          -> { context =>
        endpoints.UserTodos(Remote.record("userId" -> context.value.path("id").getOrDie))
      }
    ),
    "Post"  -> List("comments" -> { context =>
      endpoints.postComments(Remote.record("postId" -> context.value.path("id").getOrDie))
    }),
    "Album" -> List("photos" -> { context =>
      endpoints.AlbumPhotos(Remote.record("albumId" -> context.value.path("id").getOrDie))
    })
  )

  def spec = suite("OrchSpec")(test("test")(assertCompletes))
}
