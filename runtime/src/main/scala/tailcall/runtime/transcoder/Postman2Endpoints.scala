package tailcall.runtime.transcoder

import tailcall.runtime.ast.Endpoint
import tailcall.runtime.dsl.Postman
import tailcall.runtime.internal.TValid

trait Postman2Endpoints {
  def toEndpoints(
    postman: Postman,
    config: Postman2Endpoints.Config = Postman2Endpoints.Config(),
  ): TValid[String, List[Endpoint]] = ???
}

object Postman2Endpoints {
  final case class Config(allowHttpCalls: Boolean = false)
}
