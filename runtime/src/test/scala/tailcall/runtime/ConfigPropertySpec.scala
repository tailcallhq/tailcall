package tailcall.runtime

import caliban.parsing.SourceMapper
import caliban.parsing.adt.Definition.ExecutableDefinition.OperationDefinition
import caliban.parsing.adt.{Document, OperationType}
import caliban.wrappers.Wrapper.ParsingWrapper
import caliban.{CalibanError, InputValue}
import tailcall.runtime.internal.GraphQLSpec._
import tailcall.runtime.internal._
import tailcall.runtime.model.{Config, ConfigFormat}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.http.model.Headers
import zio.http.{Request, URL => ZURL}
import zio.test.Assertion.equalTo
import zio.test.TestAspect.before
import zio.test.{Spec, TestEnvironment, assertTrue, _}
import zio.{NonEmptyChunk, Scope, ZIO}

object ConfigPropertySpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = { suite("GraphQLSpec")(ZIO.flatten(makeTests("graphql"))) }

  def makeTests(dir: String) = {
    loadTests(dir).map(_.map { case ((file, graphQLSpecZio)) =>
      for {
        graphQLSpec <- graphQLSpecZio
      } yield {
        val serverSDL      = graphQLSpec.serverSDL.sdl
        val config2DocTest = List(makeConfig2DocumentTest(graphQLSpec.serverSDL.sdl))
        val clientTests    = graphQLSpec.client match {
          case Some(client) => client match {
              case sdl: Client.SDL               => List(makeConfig2ClientSDLTest(serverSDL, sdl.clientSDL.sdl)) ++
                  sdl.queries.map(query => makeExecutionTest(serverSDL, query.query, query.expectedOutput))
              case Client.ValidationError(error) => List(makeValidationTest(serverSDL, error))
            }
          case None         => Nil
        }
        val tests          = config2DocTest ::: clientTests
        suite(file.name)(tests)
      }
    }).map(s => ZIO.collectAll(s))

  }

  def makeConfig2DocumentTest(serverSDL: String) = {
    test("config2Document") {
      for {
        config <- ConfigFormat.GRAPHQL.decode(serverSDL)
        actual <- ConfigFormat.GRAPHQL.encode(config)
      } yield assertTrue(actual == serverSDL)
    }
  }

  def makeConfig2ClientSDLTest(serverSDL: String, clientSDL: String) = {
    test("config2ClientSDL") {
      for {
        config <- ConfigFormat.GRAPHQL.decode(serverSDL)
        actual <- ZIO.attempt(Transcoder.toSDL(config, false).unwrap.trim)
      } yield assertTrue(actual == clientSDL)
    }
  }

  def makeValidationTest(serverSDL: String, error: NonEmptyChunk[TValid.Cause[String]]) = {
    test("validation") {
      for {
        config <- ConfigFormat.GRAPHQL.decode(serverSDL)
        sdl    <- ZIO.attempt(Transcoder.toSDL(config, false))
      } yield assertTrue(sdl == TValid.Errors(error))
    }
  }

  private def makeExecutionTest(serverSDL: String, query: String, expectedOutput: JsonT.Constant) = {
    test("execution") {
      for {
        config  <- ConfigFormat.GRAPHQL.decode(serverSDL)
        program <- resolve(config)(query)
      } yield assert(program)(equalTo(expectedOutput.json.toString))
    }.provide(
      GraphQLGenerator.default,
      ExecutionSpecHttpClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    ) @@ before(TestSystem.putEnv("foo", "bar"))
  }

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
