package tailcall.gateway

import zio.test._
import zio.Scope
import zio.ZIO
import tailcall.gateway.Reader
import tailcall.gateway.TypeChecker
import caliban.parsing.adt.Document
import zio._

object TypeCheckerSpec extends ZIOSpecDefault {
  import caliban.parsing.Parser

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
