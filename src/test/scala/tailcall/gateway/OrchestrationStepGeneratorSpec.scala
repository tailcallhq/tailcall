package tailcall.gateway

import tailcall.gateway.ast.Orchestration
import tailcall.gateway.remote.Remote
import tailcall.gateway.service._
import zio.ZIO
import zio.schema.DynamicValue
import zio.test.Assertion.equalTo
import zio.test.{ZIOSpecDefault, assertZIO}

object OrchestrationStepGeneratorSpec extends ZIOSpecDefault {

  def spec = {
    suite("DocumentStepGenerator")(test("test") {
      val document = Orchestration(List(
        Orchestration.SchemaDefinition(query = Some("Query")),
        Orchestration.ObjectTypeDefinition(
          "Query",
          List(Orchestration.FieldDefinition(
            name = "id",
            List(),
            Orchestration.NamedType("Int", true),
            _ => Remote(DynamicValue(100))
          ))
        )
      ))

      val program = execute(document)("query {id}")

      assertZIO(program)(equalTo("""{"id":100}"""))
    }).provide(
      OrchestrationGraphQLGenerator.live,
      OrchestrationTypeGenerator.live,
      OrchestrationStepGenerator.live,
      EvaluationRuntime.live,
      EvaluationContext.live
    )
  }

  def execute(doc: Orchestration)(query: String): ZIO[OrchestrationGraphQLGenerator, Throwable, String] =
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
