package tailcall.gateway

import tailcall.gateway.ast.Document
import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.service._
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object StepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("DocumentStepGenerator")(test("test") {
      val orc = Orc("Query" -> List("id" -> Field.output.as("String").resolveWith(100)))

      val program = execute(orc)("query {id}")

      assertZIO(program)(equalTo("""{"id":100}"""))
    }).provide(
      GraphQLGenerator.live,
      TypeGenerator.live,
      StepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
  }

  def execute(orc: Orc)(query: String): ZIO[GraphQLGenerator, Throwable, String] =
    orc.toDocument.flatMap(execute(_)(query))

  def execute(doc: Document)(query: String): ZIO[GraphQLGenerator, Throwable, String] =
    for {
      graphQL     <- doc.toGraphQL
      interpreter <- graphQL.interpreter
      result <- interpreter.execute(query, skipValidation = true) // TODO: enable validation after __type is available
      _      <- result.errors.headOption match {
        case Some(error) => ZIO.fail(error)
        case None        => ZIO.unit
      }
    } yield result.data.toString
}
