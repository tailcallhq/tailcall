package tailcall.gateway

import tailcall.gateway.Reader
import zio._
import zio.test._

object TypeCheckerSpec extends ZIOSpecDefault {

  def typeCheck(configName: String, schemaName: String): Task[List[String]] =
    for {
      config   <- Reader.config.readURL(getClass.getResource(configName))
      document <- Reader.document.readURL(getClass.getResource(schemaName))
    } yield TypeChecker.check(config, document)

  override def spec =
    suite("TypeCheckerSpec")(
      test("files are being read") {
        for {
          problems <- typeCheck("Config.yml", "Schema.graphql")
        } yield assertTrue(problems == Nil)
      },
    )
}
