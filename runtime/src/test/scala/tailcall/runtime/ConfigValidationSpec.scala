package tailcall.runtime

import tailcall.runtime.internal.GraphQLTestSpec.GraphQLValidationSpec
import tailcall.runtime.internal.{GraphQLTestSpec, TValid}
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.json._
import zio.json.yaml._
import zio.test.{Spec, TestEnvironment, assertTrue}
import zio.{Scope, ZIO}

object ConfigValidationSpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = { suite("GraphQLValidationSpec")(makeTests("graphql")) }

  def makeTests(dir: String) = { loadTests[GraphQLValidationSpec](dir).map(_.map(spec => makeTest(spec))) }

  def makeTest(spec: GraphQLValidationSpec) = {
    test(spec.name) {
      val content    = spec.serverSDL
      val yamlString = removeCommentPrefix(spec.validationMessage)
      for {
        specValidationError <- ZIO.fromEither(yamlString.fromYaml[SpecValidationError])
        config              <- ConfigFormat.GRAPHQL.decode(content)
        sdl                 <- ZIO.attempt(Transcoder.toSDL(config, false))
      } yield {
        val expected = toExpectedError(specValidationError)
        assertTrue(sdl == expected)
      }
    }
  }

  def toExpectedError(specValidationError: SpecValidationError): TValid[String, String] = {
    TValid.fail(specValidationError.message).trace(specValidationError.location: _*)
  }

  implicit val decoder: JsonDecoder[SpecValidationError] = DeriveJsonDecoder.gen[SpecValidationError]
}

final case class SpecValidationError(message: String, location: List[String])
