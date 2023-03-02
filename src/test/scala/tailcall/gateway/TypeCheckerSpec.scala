package tailcall.gateway

import caliban.parsing.adt.Document
import tailcall.gateway.dsl.json.{Reader, TypeChecker}
import zio._
import zio.test._
import zio.test.TestAspect.failing

object TypeCheckerSpec extends ZIOSpecDefault:
  private def configFile: Task[dsl.json.Config] = { Reader.config.readURL(getClass.getResource("Config.yml")) }
  private def schemaFile: Task[Document]        = { Reader.document.readURL(getClass.getResource("Schema.graphql")) }

  override def spec =
    suite("TypeCheckerSpec")(test("is valid") {
      pprint.pprintln("Yo!")
      for
        config <- configFile
        schema <- schemaFile

        errors = TypeChecker.check(config, schema).errors
      yield assertTrue(errors == Chunk.empty)
    })
