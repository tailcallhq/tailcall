package tailcall.runtime

// import better.files.File
import caliban.parsing.adt.Definition.ExecutableDefinition.OperationDefinition
import caliban.parsing.adt.{Document, OperationType}
import caliban.parsing.{Parser, SourceMapper}
import caliban.wrappers.Wrapper.ParsingWrapper
import caliban.{CalibanError, InputValue}
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import tailcall.runtime.internal.{ExecutionSpecHttpClient, GraphQLTestSpec, TValid}
import tailcall.runtime.model.{Config, ConfigFormat, ExpectType}
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.http.model.Headers
import zio.http.{Request, URL => ZURL}
import zio.json._
import zio.json.ast.Json
import zio.json.yaml._
import zio.test.Assertion.equalTo
import zio.test.TestAspect.before
import zio.test.{Spec, TestEnvironment, assertTrue, _}
import zio.{Scope, ZIO}
import tailcall.runtime.internal.GraphQLConfig2DocumentSpec
import tailcall.runtime.internal.GraphQLConfig2ClientSDLSpec
import tailcall.runtime.internal.GraphQLValidationSpec
import tailcall.runtime.internal.GraphQLExecutionSpec

object ConfigPropertySpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = { suite("GraphQLSpec")(makeTests("graphQL")) }

  def makeTests(dir: String)                     = {
    loadTests(dir).map { case ((file, specList)) =>
      val tests = specList.map {
        case GraphQLConfig2DocumentSpec(serverSDL)                => makeConfig2DocumentTest(serverSDL)
        case GraphQLConfig2ClientSDLSpec(serverSDL, clientSDL)    => makeConfig2ClientSDLTest(serverSDL, clientSDL)
        case GraphQLValidationSpec(serverSDL, validationMessages) => makeValidationTest(serverSDL, validationMessages)
        case GraphQLExecutionSpec(serverSDL, query)               => makeExecutionTest(serverSDL, query)

      }
      suite(file.name)(tests)

    }
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

  def makeValidationTest(serverSDL: String, validationMessages: String) = {
    test("validation") {
      val yamlString = removeCommentPrefix(validationMessages)
      for {
        specValidationError <- ZIO.fromEither(yamlString.fromYaml[SpecValidationError])
        config              <- ConfigFormat.GRAPHQL.decode(serverSDL)
        sdl                 <- ZIO.attempt(Transcoder.toSDL(config, false))
      } yield {
        val expected = toExpectedError(specValidationError)
        assertTrue(sdl == expected)
      }
    }
  }

  private def makeExecutionTest(serverSDL: String, query: String) = {
    test("execution") {
      for {
        expected <- getExpectedOutput(query)
        config   <- ConfigFormat.GRAPHQL.decode(serverSDL)
        program  <- resolve(config)(query)
      } yield assert(program)(equalTo(expected.json.toString))
    }.provide(
      GraphQLGenerator.default,
      ExecutionSpecHttpClient.default,
      HttpContext.live(Some(Request.get(ZURL.empty).addHeaders(Headers("authorization", "bar")))),
    ) @@ before(TestSystem.putEnv("foo", "bar"))
  }

  def toExpectedError(specValidationError: SpecValidationError): TValid[String, String] = {
    TValid.fail(specValidationError.message).trace(specValidationError.location: _*)
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

  def getExpectedOutput(query: String) =
    Parser.parseQuery(query).map(document =>
      document.definitions.collect { case op: OperationDefinition => op }.flatMap { definition =>
        definition.directives.flatMap(_.fromDirective[ExpectType].toOption).headOption
      }.headOption.map(expect => expect.output).getOrElse(JsonT.Constant(Json.Obj()))
    )

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

  implicit val decoder: JsonDecoder[SpecValidationError] = DeriveJsonDecoder.gen[SpecValidationError]

}

final case class SpecValidationError(message: String, location: List[String])
