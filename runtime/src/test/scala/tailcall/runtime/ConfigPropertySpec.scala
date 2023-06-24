package tailcall.runtime

import better.files._
import tailcall.runtime.model.ConfigFormat
import tailcall.runtime.transcoder.Transcoder
import tailcall.test.TailcallSpec
import zio.test.{Gen, Spec, TestEnvironment, assertTrue, checkAll}
import zio.{Scope, ZIO}

object ConfigPropertySpec extends TailcallSpec {
  override def spec: Spec[TestEnvironment with Scope, Any] = {
    def getFiles(dir: String) = ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList)
    val sources               = Gen.fromZIO(getFiles("sources").map(Gen.fromIterable(_))).flatten

    suite("GraphQLIdentitySpec")(
      // Read .graphql files from resources
      // Perform a check from Document to Config and back to Document
      test("config to document identity") {
        checkAll(sources) { file =>
          for {
            content <- ZIO.succeedBlocking(file.contentAsString)
            expected = content.trim
            config <- ConfigFormat.GRAPHQL.decode(content)
            actual <- ConfigFormat.GRAPHQL.encode(config)
          } yield assertTrue(actual == expected)
        }
      },
      test("config to client SDL") {
        checkAll(sources) { file =>
          for {
            expectedFile <- ZIO.succeed(File(file.path.getParent.getParent.resolve("outputs").resolve(file.name)))
            expected     <- ZIO.succeedBlocking(expectedFile.contentAsString.trim)

            content <- ZIO.succeedBlocking(file.contentAsString)
            config  <- ConfigFormat.GRAPHQL.decode(content)
            actual  <- ZIO.attempt(Transcoder.toSDL(config, false).unwrap.trim)
          } yield assertTrue(actual == expected)
        }
      },
    )
  }
}
