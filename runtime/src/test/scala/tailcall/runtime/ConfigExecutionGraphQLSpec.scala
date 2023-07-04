package tailcall.runtime

import caliban.InputValue
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLExecutionSpec
import tailcall.runtime.internal.{GraphQLTestSpec, JSONPlaceholderClient}
import tailcall.runtime.model.{Config, ConfigFormat}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.http.model.Headers
import zio.http.{Request, URL => ZURL}
import zio.test.Assertion.equalTo
import zio.test.TestAspect.before
import zio.test._
import zio.{Scope, ZIO}

object ConfigExecutionGraphQLSpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("ExecutionSpec")(test("config to output") {
      checkAll(graphQLSpecGen[GraphQLExecutionSpec]("graphql")) { spec =>
        println(spec.name)
        val content  = spec.serverSDL
        val expected = removeCommentPrefix(spec.response)
        for {
          config  <- ConfigFormat.GRAPHQL.decode(content)
          program <- resolve(config)(spec.query)
        } yield assert(program)(equalTo(expected))
      }
    }).provide(
      GraphQLGenerator.default,
      JSONPlaceholderClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    ) @@ before(TestSystem.putEnv("foo", "bar"))
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
