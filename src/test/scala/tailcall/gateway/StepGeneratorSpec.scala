package tailcall.gateway

import tailcall.gateway.ast.Document
import tailcall.gateway.dsl.scala.Orc
import tailcall.gateway.dsl.scala.Orc.Field
import tailcall.gateway.remote._
import tailcall.gateway.service._
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object StepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("DocumentStepGenerator")(
      test("static value") {
        val orc     = Orc("Query" -> List("id" -> Field.output.as("String").resolveWith(100)))
        val program = execute(orc)("query {id}")
        assertZIO(program)(equalTo("""{"id":100}"""))
      },
      test("with args") {
        val orc     = Orc(
          "Query" -> List(
            "sum" -> Field.output.as("Int").withArgument("a" -> Field.input.as("Int"), "b" -> Field.input.as("Int"))
              .withResolver { ctx =>
                val a = ctx.path("args", "a").getOrDie.toTyped[Int].getOrDie
                val b = ctx.path("args", "b").getOrDie.toTyped[Int].getOrDie

                (a + b).toDynamic
              }
          )
        )
        val program = execute(orc)("query {sum(a: 1, b: 2)}")
        assertZIO(program)(equalTo("""{"sum":3}"""))
      }
    ).provide(
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
