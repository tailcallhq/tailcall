package tailcall.registry

import tailcall.runtime.model.Config
import zio.Scope
import zio.test.Assertion.{equalTo, isSome}
import zio.test._

object SchemaRegistrySpec extends ZIOSpecDefault {
  val config = Config.default.withTypes(
    "Query" -> Config.Type(
      "name" -> Config.Field.ofType("String").resolveWithJson("John Doe"),
      "age"  -> Config.Field.ofType("Int").resolveWithJson(100),
    )
  )

  val registrySpec = test("set & get") {
    for {
      blueprint <- config.toBlueprint.toTask
      digest    <- SchemaRegistry.add(blueprint)
      actual    <- SchemaRegistry.get(digest)
    } yield assert(actual)(isSome(equalTo(blueprint)))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("SchemaRegistrySpec")(
      suite("In Memory")(registrySpec).provide(SchemaRegistry.memory),
      suite("My SQL")(registrySpec).provide(SchemaRegistry.mysql("localhost", 3306, Option("root"), Option("1234"))),
    )
  }
}
