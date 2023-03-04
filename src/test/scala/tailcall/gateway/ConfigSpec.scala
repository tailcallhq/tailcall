package tailcall.gateway

import tailcall.gateway.ast.{Path, TSchema}
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config.Step
import tailcall.gateway.internal.Extension
import zio.test.{ZIOSpecDefault, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
  override def spec =
    suite("ConfigSpec")(test("encoding") {

      val getUsers    = Config.Step.Http(Path.unsafe.fromString("/users"))
      val getPosts    = Config.Step.Http(Path.unsafe.fromString("/posts"))
      val getComments = Config.Step.Http(Path.unsafe.fromString("/comments"))
      val getAlbums   = Config.Step.Http(Path.unsafe.fromString("/albums"))
      val getPhotos   = Config.Step.Http(Path.unsafe.fromString("/photos"))

      val userPosts: Step    = Config.Step.Http(Path.unsafe.fromString("/users/{{id}}/posts"))
      val userComments: Step = Config.Step.Http(Path.unsafe.fromString("/users/{{id}}/comments"))
      val userAlbums: Step   = Config.Step.Http(Path.unsafe.fromString("/users/{{id}}/albums"))

      val postComments: Step = Config.Step.Http(Path.unsafe.fromString("/posts/{{id}}/comments"))
      val postUser: Step     = Config.Step.Http(Path.unsafe.fromString("/posts/{{id}}/user"))

      val commentPost = Config.Step.Http(Path.unsafe.fromString("/comments/{{id}}/post"))
      val photoAlbum  = Config.Step.Http(Path.unsafe.fromString("/photos/{{id}}/album"))

      val graphQL = Config.GraphQL(
        schema = Config.SchemaDefinition(query = Some("Query"), mutation = Some("Mutation")),
        types = Map(
          "Query"   -> Map(
            "user"     -> Config.Field("User", getUsers)("id" -> TSchema.str),
            "users"    -> Config.Field("User", getUsers).withList,
            "posts"    -> Config.Field("Post", getPosts),
            "comments" -> Config.Field("Comment", getComments),
            "albums"   -> Config.Field("Album", getAlbums),
            "photos"   -> Config.Field("Photo", getPhotos)
          ),
          "User"    ->
            Map(
              "id"       -> Config.Field("String"),
              "name"     -> Config.Field("String"),
              "email"    -> Config.Field("String"),
              "address"  -> Config.Field("Address"),
              "phone"    -> Config.Field("String"),
              "website"  -> Config.Field("String"),
              "company"  -> Config.Field("Company"),
              "posts"    -> Config.Field("Post", userPosts),
              "comments" -> Config.Field("Comment", userComments),
              "albums"   -> Config.Field("Album", userAlbums)
            ),
          "Post"    -> Map(
            "id"       -> Config.Field("String"),
            "userId"   -> Config.Field("Id"),
            "title"    -> Config.Field("String"),
            "body"     -> Config.Field("String"),
            "user"     -> Config.Field("User", postUser),
            "comments" -> Config.Field("Comment", postComments)
          ),
          "Address" -> Map(
            "street"  -> Config.Field("String"),
            "suite"   -> Config.Field("String"),
            "city"    -> Config.Field("String"),
            "zipcode" -> Config.Field("String"),
            "geo"     -> Config.Field("Geo")
          ),
          "Company" -> Map(
            "name"        -> Config.Field("String"),
            "catchPhrase" -> Config.Field("String"),
            "bs"          -> Config.Field("String")
          ),
          "Geo"     -> Map("lat" -> Config.Field("String"), "lng" -> Config.Field("String")),
          "Comment" -> Map(
            "id"     -> Config.Field("String"),
            "postId" -> Config.Field("Id"),
            "name"   -> Config.Field("String"),
            "email"  -> Config.Field("String"),
            "body"   -> Config.Field("String"),
            "post"   -> Config.Field("Post", commentPost)
          ),
          "Photo"   -> Map(
            "id"           -> Config.Field("String"),
            "albumId"      -> Config.Field("Id"),
            "title"        -> Config.Field("String"),
            "url"          -> Config.Field("String"),
            "thumbnailUrl" -> Config.Field("String"),
            "album"        -> Config.Field("Album", photoAlbum)
          )
        )
      )

      val server = Config.Server("https://jsonplaceholder.typicode.com/")
      val config = Config(server = server, graphQL = graphQL)

      val extension = Extension.YML

      for {
        encoded <- extension.encode(config)
        _ = pprint.pprintln(encoded)
        decoded <- extension.decode(encoded)
      } yield assertTrue(decoded == config)
    })
}
