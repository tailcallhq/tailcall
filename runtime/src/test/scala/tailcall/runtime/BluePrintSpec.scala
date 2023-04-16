package tailcall.runtime

import tailcall.runtime.http.Method
import tailcall.runtime.lambda.Lambda
import tailcall.runtime.model.{Blueprint, Endpoint}
import zio.test._

object BluePrintSpec extends ZIOSpecDefault {
  override def spec =
    suite("blueprint")(test("encode") {
      val endpoint      = Endpoint.make("abc.com").withMethod(Method.GET)
      val resolver      = Lambda.unsafe.fromEndpoint(endpoint)
      val blueprint     = Blueprint(List(Blueprint.ObjectTypeDefinition(
        "Query",
        List(
          Blueprint
            .FieldDefinition(name = "health", ofType = Blueprint.NamedType("string", true), resolver = Some(resolver))
        ),
      )))
      val blueprintJson = Blueprint.encode(blueprint)
      val decoded       = Blueprint.decode(blueprintJson)
      assertTrue(decoded == Right(blueprint))
    })
}
