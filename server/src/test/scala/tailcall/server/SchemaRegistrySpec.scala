package tailcall.server
import tailcall.runtime.dsl.scala.Orc
import tailcall.runtime.dsl.scala.Orc.FieldSet
import tailcall.server.service.SchemaRegistry
import zio.Scope
import zio.test.Assertion.{equalTo, isSome}
import zio.test._

object SchemaRegistrySpec extends ZIOSpecDefault {

  val orc = Orc(
    "Query" -> FieldSet(
      "name" -> Orc.Field.output.to("String").resolveWith("John Doe"),
      "age"  -> Orc.Field.output.to("Int").resolveWith(100)
    )
  )

  val registrySpec = test("set & get") {
    for {
      blueprint <- orc.toBlueprint
      digest    <- SchemaRegistry.add(blueprint)
      actual    <- SchemaRegistry.get(digest)
    } yield assert(actual)(isSome(equalTo(blueprint)))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("SchemaRegistrySpec")(
      suite("InMemory")(registrySpec).provide(SchemaRegistry.memory),
      suite("Persistent")(registrySpec).provide(SchemaRegistry.persistent)
    )
  }
}
