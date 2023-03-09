package tailcall.runtime

import tailcall.runtime.dsl.json.Reader
import zio.test.TestAspect.timeout
import zio.test._
import zio.{Scope, durationInt}

object ReaderSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("Reader")(
      test("Config.yml is valid")(Reader.config.readURL(getClass.getResource("Config.yml")).as(assertCompletes)),
      test("Config.json is valid")(Reader.config.readURL(getClass.getResource("Config.json")).as(assertCompletes)),
      test("Schema.graphql is valid")(
        Reader.document.readURL(getClass.getResource("Schema.graphql")).as(assertCompletes)
      )
    ) @@ timeout(5 seconds)
}
