package tailcall.runtime.internal

import tailcall.runtime.http.{HttpClient, Method, Request}
import zio.http.Response
import zio.{ZIO, ZLayer}

class ExecutionSpecHttpClient() extends HttpClient {

  override def allowedHeaders: Set[String] = Set.empty

  def simpleQuery: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("\"Hello World\""))

  def inlineField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":{"b":{"c":"Hello"}}}"""))

  def inlineFieldScalarType: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":"Hello"}"""))

  def inlineIndexList: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":[{"b":[{"c":"Hello"}]}]}"""))

  def inlineWithList: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":[{"b":[{"c":"Hello"}]}]}"""))

  def inlineWithModifyField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":{"b":{"c":"Hello"}}}"""))

  def nestedType: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"b":[{"c":1},{"c":2},{"c":3}]}"""))

  def renameArgument: ZIO[Any, Throwable, Response] = { ZIO.succeed(Response.json("1")) }

  def renameField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json(""""Hello World""""))

  def resolvedByParent: ZIO[Any, Throwable, Response] =
    ZIO.succeed(Response.json("""{"address":{"street": "James Street"}}"""))

  def dictionary: ZIO[Any, Throwable, Response] =
    ZIO.succeed(Response.json("""{"a":1,"b":[{"key":"k1","value":1},{"key":"k2","value":2},{"key":"k3","value":3}]}"""))

  override def request(req: Request): ZIO[Any, Throwable, Response] =
    (req.url, req.method) match {
      case (url, Method.GET) if url == "https://foo.com/simpleQuery"           => simpleQuery
      case (url, Method.GET) if url == "https://foo.com/dictionary"            => dictionary
      case (url, Method.GET) if url == "https://foo.com/inlineFieldScalarType" => inlineFieldScalarType
      case (url, Method.GET) if url == "https://foo.com/inlineField"           => inlineField
      case (url, Method.GET) if url == "https://foo.com/inlineIndexList"       => inlineIndexList
      case (url, Method.GET) if url == "https://foo.com/inlineWithList"        => inlineWithList
      case (url, Method.GET) if url == "https://foo.com/inlineWithModifyField" => inlineWithModifyField
      case (url, Method.GET) if url == "https://foo.com/nestedType"            => nestedType
      case (url, Method.GET) if url == "https://foo.com/renameArgument"        => renameArgument
      case (url, Method.GET) if url == "https://foo.com/renameField"           => renameField
      case (url, Method.GET) if url == "https://foo.com/resolvedByParent"      => resolvedByParent

      case _ => ZIO.fail(new IllegalArgumentException(s"Invalid request: $req"))
    }
}

object ExecutionSpecHttpClient {
  def default: ZLayer[Any, Throwable, ExecutionSpecHttpClient] = ZLayer.succeed(new ExecutionSpecHttpClient())
}
