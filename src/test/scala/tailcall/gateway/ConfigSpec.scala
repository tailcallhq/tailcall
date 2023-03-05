package tailcall.gateway

import tailcall.gateway.internal.{Extension, JsonPlaceholderConfig}
import tailcall.gateway.service.{EvaluationRuntime, GraphQLGenerator, StepGenerator, TypeGenerator}
import zio.test.{ZIOSpecDefault, assertCompletes, assertTrue}

object ConfigSpec extends ZIOSpecDefault {
  override def spec =
    suite("ConfigSpec")(
      test("encoding") {
        val extension = Extension.YML
        val config    = JsonPlaceholderConfig.config
        for {
          encoded <- extension.encode(config)
          decoded <- extension.decode(encoded)
        } yield assertTrue(decoded == config)
      },
      test("render") {
        val config = JsonPlaceholderConfig.config
        for {
          graphQL <- config.toBlueprint.toGraphQL
          _ = pprint.pprintln(graphQL.render)
        } yield assertCompletes
      }
    ).provide(GraphQLGenerator.live, TypeGenerator.live, StepGenerator.live, EvaluationRuntime.live)
}
