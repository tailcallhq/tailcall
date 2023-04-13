package tailcall.registry

import tailcall.runtime.model.Config
import zio.Scope
import zio.test.Assertion.{equalTo, isSome}
import zio.test._

object SchemaRegistrySpec extends ZIOSpecDefault {

  val config = Config.default.withTypes(
    "Query" -> Config.Type(
      "name" -> Config.Field.ofType("String").resolveWith("John Doe"),
      "age"  -> Config.Field.ofType("Int").resolveWith(100),
    )
  )

  val registrySpec = test("set & get") {
    val blueprint = config.toBlueprint
    for {
      digest <- SchemaRegistry.add(blueprint)
      actual <- SchemaRegistry.get(digest)
    } yield assert(actual)(isSome(equalTo(blueprint)))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("SchemaRegistrySpec")(suite("InMemory")(registrySpec).provide(SchemaRegistry.memory))
  }
}
