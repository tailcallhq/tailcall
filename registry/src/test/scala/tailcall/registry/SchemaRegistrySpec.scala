package tailcall.registry

import tailcall.runtime.model.Config
import tailcall.test.TailcallSpec
import zio.test.Assertion.{equalTo, isNone, isSome}
import zio.test.TestAspect.{ignore, sequential}
import zio.test._
import zio.{Scope, ZIO}

object SchemaRegistrySpec extends TailcallSpec {
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
        actual    <- SchemaRegistry.get(digest.hex)
      } yield assert(actual)(isSome(equalTo(blueprint)))
    },
    test("add multiple times") {
      for {
        blueprint <- config.toBlueprint.toTask
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.add(blueprint)
        actual    <- SchemaRegistry.get(blueprint.digest.hex)
      } yield assert(actual)(isSome(equalTo(blueprint)))
    },
    test("add multiple times in parallel") {
      for {
        blueprint <- config.toBlueprint.toTask
        _         <- ZIO.foreachParDiscard(1 to 2)(_ => SchemaRegistry.add(blueprint))
        actual    <- SchemaRegistry.list(0, 10)
      } yield assert(actual)(equalTo(List(blueprint)))

      // TODO: need to debug why it fails for MYSQL
    } @@ ignore,
    test("drop") {
      for {
        blueprint <- config.toBlueprint.toTask
        _         <- SchemaRegistry.add(blueprint)
        _         <- SchemaRegistry.drop(blueprint.digest.hex)
        actual    <- SchemaRegistry.get(blueprint.digest.hex)
      } yield assert(actual)(isNone)
    },
    suite("short sha")(
      test("set & get") {
        for {
          blueprint <- config.toBlueprint.toTask
          digest    <- SchemaRegistry.add(blueprint)
          actual    <- SchemaRegistry.get(digest.prefix)
        } yield assert(actual)(isSome(equalTo(blueprint)))
      },
      test("add multiple times") {
        for {
          blueprint <- config.toBlueprint.toTask
          _         <- SchemaRegistry.add(blueprint)
          _         <- SchemaRegistry.add(blueprint)
          _         <- SchemaRegistry.add(blueprint)
          actual    <- SchemaRegistry.get(blueprint.digest.prefix)
        } yield assert(actual)(isSome(equalTo(blueprint)))
      },
      test("drop by short sha") {
        for {
          blueprint <- config.toBlueprint.toTask
          _         <- SchemaRegistry.add(blueprint)
          _         <- SchemaRegistry.drop(blueprint.digest.prefix)
          actual    <- SchemaRegistry.get(blueprint.digest.prefix)
        } yield assert(actual)(isNone)
      },
    ),
  )

  override def spec: Spec[TestEnvironment with Scope, Any] = {
    suite("SchemaRegistrySpec")(
      suite("In Memory")(registrySpec).provide(SchemaRegistry.memory),
      suite("My SQL")(registrySpec @@ sequential)
        .provide(SchemaRegistry.mysql("localhost", 3306, Option("tailcall_main_user"), Option("tailcall"))),
    )
  }
}
