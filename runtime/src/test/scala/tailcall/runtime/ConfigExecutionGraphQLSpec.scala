package tailcall.runtime

import caliban.parsing.adt.Definition.ExecutableDefinition.OperationDefinition
import caliban.parsing.adt.{Document, OperationType}
import caliban.parsing.{Parser, SourceMapper}
import caliban.wrappers.Wrapper.ParsingWrapper
import caliban.{CalibanError, InputValue}
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLExecutionSpec
import tailcall.runtime.internal.{ExecutionSpecHttpClient, GraphQLTestSpec}
import tailcall.runtime.model.{Config, ConfigFormat, ExpectType}
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
    suite("ExecutionSpec")(makeTests("graphql")).provide(
      GraphQLGenerator.default,
      ExecutionSpecHttpClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    ) @@ before(TestSystem.putEnv("foo", "bar"))
  }

  private def makeTests(dir: String) = { loadTests[GraphQLExecutionSpec](dir).map(_.map(spec => makeTest(spec))) }

  def removeDirectivesFromQuery(): ParsingWrapper[Any] =
    new ParsingWrapper[Any] {
      def wrap[R1](
        process: String => ZIO[R1, CalibanError.ParsingError, Document]
      ): String => ZIO[R1, CalibanError.ParsingError, Document] =
        (query: String) =>
          process(query).map(document => {
            val newDefinitions = document.definitions.map {
              case op: OperationDefinition if op.operationType == OperationType.Query =>
                op.copy(directives = List.empty)
              case other                                                              => other
            }
            document.copy(definitions = newDefinitions, sourceMapper = SourceMapper.empty)
          })
    }

  def getExpectedOutput(query: String) =
    Parser.parseQuery(query).map(document =>
      document.definitions.collect { case op: OperationDefinition => op }.flatMap { definition =>
        definition.directives.flatMap(_.fromDirective[ExpectType].toOption).headOption
      }.headOption.map(expect => expect.output).getOrElse("")
    )

  private def makeTest(spec: GraphQLExecutionSpec) = {
    test(spec.name) {
      val content = spec.serverSDL
      for {
        expected <- getExpectedOutput(spec.query)
        config   <- ConfigFormat.GRAPHQL.decode(content)
        program  <- resolve(config)(spec.query)
      } yield assert(program)(equalTo(expected))
    }
  }

  private def resolve(config: Config, variables: Map[String, InputValue] = Map.empty)(
    query: String
  ): ZIO[HttpContext with GraphQLGenerator, Throwable, String] = {
    for {
      blueprint   <- Transcoder.toBlueprint(config).toTask
      graphQL     <- blueprint.toGraphQL.map(graphQL => graphQL.withWrapper(removeDirectivesFromQuery()))
      interpreter <- graphQL.interpreter
      result      <- interpreter.execute(query, variables = variables)
      _           <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
  }

}
