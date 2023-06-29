package tailcall.runtime

import tailcall.runtime.internal.{GraphQLTestSpec, TValid}
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.test.{Spec, TestEnvironment, assertTrue, checkAll}
import zio.{Scope, ZIO}

import scala.collection.immutable.ArraySeq.unsafeWrapArray

object ConfigPropertySpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("GraphQLIdentitySpec")(
      // Read .graphql files from resources
      // Perform a check from Document to Config and back to Document
      test("config to document identity") {
        checkAll(graphQLSpecGen("graphql")) { spec =>
          val content  = spec.serverSDL
          val expected = content
          for {
            config <- ConfigFormat.GRAPHQL.decode(content)
            actual <- ConfigFormat.GRAPHQL.encode(config)
          } yield assertTrue(actual == expected)
        }
      },
      test("config to client SDL") {
        checkAll(graphQLSpecGen("graphql")) { spec =>
          val expected = if (spec.clientError.isBlank) spec.clientSDL else buildExpectedError(spec.clientError)
          val content  = spec.serverSDL
          for {
            config <- ConfigFormat.GRAPHQL.decode(content)
            sdl    <- ZIO.attempt(Transcoder.toSDL(config, false))
          } yield {
            val actual = if (sdl.isValid) sdl.unwrap.trim else sdl
            assertTrue(actual == expected)
          }
        }
      },
    )
  }

  def buildExpectedError(inputString: String): TValid[String, String] = {
    val parts    = inputString.split("] ")
    val message  = parts(1)
    val location = parts(0).replace("# - [", "").split(",").map(_.trim)
    TValid.fail(message).trace(unsafeWrapArray(location): _*)
  }
}
