package tailcall.runtime

import tailcall.runtime.service.{ConfigFileReader, FileIO, GraphQLFileReader}
import zio.test.TestAspect.timeout
import zio.test._
import zio.{Scope, durationInt}

object ConfigFileReaderSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("Reader")(
      test("Config.yml is valid")(ConfigFileReader.readURL(getClass.getResource("Config.yml")).as(assertCompletes)),
      test("Config.json is valid")(ConfigFileReader.readURL(getClass.getResource("Config.json")).as(assertCompletes)),
      test("Schema.graphql is valid")(
        GraphQLFileReader.readURL(getClass.getResource("Schema.graphql")).as(assertCompletes)
      )
    ).provide(GraphQLFileReader.live, ConfigFileReader.live, FileIO.live) @@ timeout(5 seconds)
}
