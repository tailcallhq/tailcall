package tailcall.runtime

import tailcall.TailcallSpec
import tailcall.runtime.internal.GraphQLTestSpec
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.transcoder.Transcoder
import zio.test.{Spec, TestEnvironment, assertTrue, checkAll}
import zio.{Scope, ZIO}

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
          val expected = spec.clientSDL
          val content  = spec.serverSDL
          for {
            config <- ConfigFormat.GRAPHQL.decode(content)
            actual <- ZIO.attempt(Transcoder.toSDL(config, false).unwrap.trim)
          } yield assertTrue(actual == expected)
        }
      },
    )
  }
}
