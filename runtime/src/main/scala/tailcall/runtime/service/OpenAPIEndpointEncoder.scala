package tailcall.runtime.service

import tailcall.runtime.ast.Endpoint
import zio.http.api.openapi.OpenAPI.OpenAPI
import zio.{ZIO, ZLayer}

trait OpenAPIEndpointEncoder {
  def encode(api: OpenAPI): List[Endpoint]
}

object OpenAPIEndpointEncoder {
  def encode(api: OpenAPI): ZIO[OpenAPIEndpointEncoder, Nothing, List[Endpoint]] = ZIO.serviceWith(_.encode(api))

  def live: ZLayer[Any, Nothing, OpenAPIEndpointEncoder] = ZLayer.succeed(new Live)

  final class Live extends OpenAPIEndpointEncoder {
    def encode(api: OpenAPI): List[Endpoint] = ???
  }
}
