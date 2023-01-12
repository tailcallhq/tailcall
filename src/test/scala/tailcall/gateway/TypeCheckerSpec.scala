package tailcall.gateway

import caliban.parsing.adt.Document
import tailcall.gateway.Reader
import zio._
import zio.test._

object TypeCheckerSpec extends ZIOSpecDefault {
  private val configFile: Task[adt.Config] = {
    Reader.config.readURL(getClass.getResource("Config.yml"))
  }

  private val schemaFile: Task[Document] = {
    Reader.document.readURL(getClass.getResource("Schema.graphql"))
  }

  override def spec =
    suite("TypeCheckerSpec")(
      test("is valid") {
        for {
          config <- configFile
          schema <- schemaFile
          errors = TypeChecker.check(config, schema).errors
        } yield assertTrue(errors == Chunk.empty)
      },
    )
}
