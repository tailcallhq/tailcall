package tailcall.runtime

import tailcall.runtime.internal.GraphQLTestSpec
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLSDLSpec
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.test.{Spec, TestEnvironment, assertTrue}
import zio.{Scope, ZIO}

object ConfigPropertySpec extends TailcallSpec with GraphQLTestSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    val tests = loadTests[GraphQLSDLSpec]("graphql")
    suite("GraphQLIdentitySpec")(
      suite("GraphQLConfig2DocumentSpec")(tests.map(_.map(spec => makeConfig2DocumentTest(spec)))),
      suite("GraphQLConfig2ClientSDLSpec")(tests.map(_.map(spec => makeConfig2ClientSDLTest(spec)))),
    )
  }

  def makeTests(dir: String) = {
    loadTests[GraphQLSDLSpec](dir)
      .map(_.map(spec => List(makeConfig2DocumentTest(spec), makeConfig2ClientSDLTest(spec))).flatten)
  }

  def makeConfig2DocumentTest(spec: GraphQLSDLSpec) = {
    test(spec.name) {
      val content  = spec.serverSDL
      val expected = content
      for {
        config <- ConfigFormat.GRAPHQL.decode(content)
        actual <- ConfigFormat.GRAPHQL.encode(config)
      } yield assertTrue(actual == expected)
    }
  }

  def makeConfig2ClientSDLTest(spec: GraphQLSDLSpec) = {
    test(spec.name) {
      val expected = spec.clientSDL
      val content  = spec.serverSDL
      for {
        config <- ConfigFormat.GRAPHQL.decode(content)
        actual <- ZIO.attempt(Transcoder.toSDL(config, false).unwrap.trim)
      } yield assertTrue(actual == expected)
    }
  }

}
