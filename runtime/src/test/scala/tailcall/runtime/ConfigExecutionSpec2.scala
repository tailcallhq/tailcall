package tailcall.runtime

import caliban.InputValue
// import tailcall.runtime.internal.JSONPlaceholderClient
import tailcall.runtime.internal.GraphQLTestSpec
import tailcall.runtime.model.Config
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.http.{Request, URL => ZURL}
import zio.http.model.Headers
// import zio.schema.{DynamicValue, Schema}
import zio.test.{Spec, TestEnvironment, checkAll, assertZIO}
import zio.test.Assertion.equalTo
// import zio.test.TestAspect.before
import zio.{Scope, ZIO}
// import zio.{Scope, UIO, ZIO, Ref}

object ConfigExecutionSpec2 extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("ExecutionSpec")(test("config to output") {

      checkAll(graphQLSpecGen("graphql")) { spec =>
        // val expected = spec.clientSDL
        val content = spec.serverSDL
        for {
          config <- ConfigFormat.GRAPHQL.decode(content)
          // actual <- ZIO.attempt(Transcoder.toSDL(config, false).unwrap.trim)
        } yield {
          println(config.outputTypes)
          println(spec.clientQuery)
          val program = resolve(config)(spec.clientQuery)
          assertZIO(program)(equalTo("""Hello World"""))
        }
      }

    }).provide(
      GraphQLGenerator.default,
      // JSONPlaceholderClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    )

  }

  private def resolve(config: Config, variables: Map[String, InputValue] = Map.empty)(
    query: String
  ): ZIO[HttpContext with GraphQLGenerator, Throwable, String] = {
    for {
      blueprint   <- Transcoder.toBlueprint(config).toTask
      graphQL     <- blueprint.toGraphQL
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query, variables = variables)
      _           <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
  }

}
