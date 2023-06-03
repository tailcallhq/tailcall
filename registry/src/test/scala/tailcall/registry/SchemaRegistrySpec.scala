package tailcall.registry

import tailcall.runtime.model.Config
import zio.Scope
import zio.test.Assertion.{equalTo, isNone, isSome}
import zio.test.TestAspect.sequential
import zio.test._

object SchemaRegistrySpec extends ZIOSpecDefault {
  private val config = Config.default.withTypes(
    "Query" -> Config.Type(
      "name" -> Config.Field.ofType("String").resolveWithJson("John Doe"),
      "age"  -> Config.Field.ofType("Int").resolveWithJson(100),
    )
  )

  private val registrySpec = suite("RegistrySpec")(
    test("set & get") {
      for {
        blueprint <- config.toBlueprint.toTask
        digest    <- SchemaRegistry.add(blueprint)
        actual    <- SchemaRegistry.get(digest)
      } yield assert(actual)(isSome(equalTo(blueprint)))
    },
    test("add multiple times") {
      for {
        blueprint <- config.toBlueprint.toTask
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.add(blueprint)
        actual    <- SchemaRegistry.get(blueprint.digest)
      } yield assert(actual)(isSome(equalTo(blueprint)))
    },
    test("drop") {
      for {
        blueprint <- config.toBlueprint.toTask
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.drop(blueprint.digest)
        actual    <- SchemaRegistry.get(blueprint.digest)
      } yield assert(actual)(isNone)
    },
  )

  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("SchemaRegistrySpec")(
      suite("In Memory")(registrySpec).provide(SchemaRegistry.memory),
      suite("My SQL")(registrySpec @@ sequential)
        .provide(SchemaRegistry.mysql("localhost", 3306, Option("tailcall_main_user"), Option("tailcall"))),
    )
  }
}
