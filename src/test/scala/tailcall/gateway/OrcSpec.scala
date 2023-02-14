package tailcall.gateway

import tailcall.gateway.ast.Orc
import tailcall.gateway.remote.Remote
import zio.test._

object OrcSpec extends ZIOSpecDefault {

  def spec =
    suite("OrcSpec")(test("test") {
      val orc     = Orc.query("Query" -> List("count" -> { _ =>
        Remote.dynamicValue(1)
      }))
      val graphQL = orc.toGraphQLSchema
      pprint.pprintln(graphQL)
      assertCompletes
    })
}
