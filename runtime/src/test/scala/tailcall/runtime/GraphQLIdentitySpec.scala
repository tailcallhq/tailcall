package tailcall.runtime

import better.files._
import tailcall.runtime.model.ConfigFormat
import tailcall.test.TailcallSpec
import zio.test.{Gen, Spec, TestEnvironment, assertTrue, checkAll}
import zio.{Scope, ZIO}

object GraphQLIdentitySpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("GraphQLIdentitySpec")(
      // Read .graphql files from resources
      // Perform a check from Document to Config and back to Document
      test("config to document identity") {
        val files = Gen.fromZIO {
          ZIO.succeed(File(getClass.getResource("sources")).glob("*.graphql").toList).map(Gen.fromIterable(_))
        }.flatten

        checkAll(files) { file =>
          for {
            content <- ZIO.succeed(file.contentAsString)
            expected = content.trim
            config <- ConfigFormat.GRAPHQL.decode(content)
            actual <- ConfigFormat.GRAPHQL.encode(config)
          } yield assertTrue(actual == expected)
        }
      }
    )
}
