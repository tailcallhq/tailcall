package tailcall.runtime

import tailcall.runtime.model.UnsafeSteps.Operation.Http
import tailcall.runtime.model.{Config, Path, Server}
import zio.test._

import java.net.URL

object Config2BlueprintSpec extends ZIOSpecDefault {
  def spec =
    suite("Config2BlueprintSpec")(
      test("timeout") {
        val timeout = Config(server = Server(baseURL = Some(new URL("http://localhost:8080")), timeout = Some(1000)))
          .toBlueprint.toOption.flatMap(_.server.globalResponseTimeout)

        assertTrue(timeout == Option(1000))
      },
      test("cyclic types") {
        val config = Config.default.withBaseURL(new URL("https://jsonplaceholder.com")).withTypes(
          "Query" -> Config.Type("users" -> Config.Field.ofType("User").asList),
          "User"  -> Config.Type(
            "name"  -> Config.Field.string,
            "id"    -> Config.Field.int,
            "posts" -> Config.Field.ofType("Post").asList.withHttp(Http(path = Path.unsafe.fromString("/posts"))),
          ),
          "Post"  -> Config.Type(
            "name" -> Config.Field.string,
            "id"   -> Config.Field.int,
            "user" -> Config.Field.ofType("User").withHttp(Http(path = Path.unsafe.fromString("/users"))),
          ),
        )

        val blueprint = config.toBlueprint.toEither

        assertTrue(blueprint.isRight)
      },
    )
}
