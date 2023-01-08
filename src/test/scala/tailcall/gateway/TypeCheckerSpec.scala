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

  def typeCheck(config: String, schema: String): Task[TypeChecker.Status] =
    for {
      config   <- Reader.config.readURL(getClass.getResource(config))
      document <- Reader.document.readURL(getClass.getResource(schema))
    } yield TypeChecker.Status.Empty

  override def spec =
    suite("TypeCheckerSpec")(
      test("files are being read") {
        for {
          status <- typeCheck("Config.yml", "Schema.graphql")
        } yield assertTrue(status == TypeChecker.Status.Empty)
      },
    )
}
