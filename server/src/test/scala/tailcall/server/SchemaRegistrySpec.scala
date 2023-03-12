package tailcall.server
import tailcall.runtime.dsl.scala.Orc
import tailcall.runtime.dsl.scala.Orc.FieldSet
import tailcall.server.service.{BinaryDigest, SchemaRegistry}
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
    val path = this.getClass.getResource("/").getPath
    suite("SchemaRegistrySpec")(
      suite("InMemory")(registrySpec).provide(SchemaRegistry.memory, BinaryDigest.sha256),
      suite("Persistent")(registrySpec).provide(SchemaRegistry.persistent(path), BinaryDigest.sha256)
    )
  }
}
