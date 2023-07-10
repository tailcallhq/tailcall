package tailcall.runtime.internal

import tailcall.runtime.http.{HttpClient, Method, Request}
import zio.http.Response
import zio.{ZIO, ZLayer}

import java.net.URI

class ExecutionSpecHttpClient() extends HttpClient {

  override def allowedHeaders: Set[String] = Set.empty

  def simpleQuery: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("\"Hello World\""))

  def inlineField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":{"b":{"c":"Hello"}}}"""))

  def inlineFieldScalarType: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":"Hello"}"""))

  def inlineIndexList: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":[{"b":[{"c":"Hello"}]}]}"""))

  def inlineWithList: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":[{"b":[{"c":"Hello"}]}]}"""))

  def inlineWithModifyField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"a":{"b":{"c":"Hello"}}}"""))

  def nestedType: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"b":[{"c":1},{"c":2},{"c":3}]}"""))

  def renameArgument(req: Request): ZIO[Any, Throwable, Response] = {
    val data = new URI(req.url).getQuery().split("=")(1)
    ZIO.succeed(Response.json(s"{\"bar\":${data}}"))
  }

  def renameField: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json(""""Hello World""""))

  def resolvedByParent: ZIO[Any, Throwable, Response] =
    ZIO.succeed(Response.json("""{"address":{"street": "James Street"}}"""))

  def dictionary: ZIO[Any, Throwable, Response] =
    ZIO.succeed(Response.json("""{"a":1,"b":[{"key":"k1","value":1},{"key":"k2","value":2},{"key":"k3","value":3}]}"""))

  def nestedObjects: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("""{"bar":"Hello World"}"""))

  def staticValue: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("100"))

  def withArgs(req: Request): ZIO[Any, Throwable, Response] = {
    val query  = new URI(req.url).getQuery()
    val params = query.split("&")
    val a      = params(0).split("=")(1).toInt
    val b      = params(1).split("=")(1).toInt
    ZIO.succeed(Response.json(s"${a + b}"))
  }

  def withNesting: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("100"))

  def withNestingArrayBar: ZIO[Any, Throwable, Response]   = ZIO.succeed(Response.json("[100,200,300]"))
  def withNestingArrayValue: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("100"))

  def withNestingArrayCtxBar: ZIO[Any, Throwable, Response] = ZIO.succeed(Response.json("[100,200,300]"))
  def withNestingArrayCtxValue(req: Request): ZIO[Any, Throwable, Response] = {
    val data = new URI(req.url).getQuery().split("=")(1)
    ZIO.succeed(Response.json(s"${data.toInt + 1}"))
  }

  def withNestingLevel3Bar: ZIO[Any, Throwable, Response]                 = ZIO.succeed(Response.json("[100,200,300]"))
  def withNestingLevel3Baz(req: Request): ZIO[Any, Throwable, Response]   = {
    val data = new URI(req.url).getQuery().split("=")(1)
    ZIO.succeed(Response.json(s"${data.toInt + 1}"))
  }
  def withNestingLevel3Value(req: Request): ZIO[Any, Throwable, Response] = {
    val data = new URI(req.url).getQuery().split("=")(1)
    ZIO.succeed(Response.json(s"${data.toInt + 1}"))
  }

  override def request(req: Request): ZIO[Any, Throwable, Response] =
    (req.url, req.method) match {
      case (url, Method.GET) if url == "https://foo.com/simpleQuery"                       => simpleQuery
      case (url, Method.GET) if url == "https://foo.com/dictionary"                        => dictionary
      case (url, Method.GET) if url == "https://foo.com/inlineFieldScalarType"             => inlineFieldScalarType
      case (url, Method.GET) if url == "https://foo.com/inlineField"                       => inlineField
      case (url, Method.GET) if url == "https://foo.com/inlineIndexList"                   => inlineIndexList
      case (url, Method.GET) if url == "https://foo.com/inlineWithList"                    => inlineWithList
      case (url, Method.GET) if url == "https://foo.com/inlineWithModifyField"             => inlineWithModifyField
      case (url, Method.GET) if url == "https://foo.com/nestedType"                        => nestedType
      case (url, Method.GET) if url == "https://foo.com/renameArgument?data=1"             => renameArgument(req)
      case (url, Method.GET) if url == "https://foo.com/renameField"                       => renameField
      case (url, Method.GET) if url == "https://foo.com/resolvedByParent"                  => resolvedByParent
      case (url, Method.GET) if url == "https://foo.com/nestedObjects"                     => nestedObjects
      case (url, Method.GET) if url == "https://foo.com/staticValue"                       => staticValue
      case (url, Method.GET) if url == "https://foo.com/withArgs?a=1&b=2"                  => withArgs(req)
      case (url, Method.GET) if url == "https://foo.com/withNesting"                       => withNesting
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayBar"               => withNestingArrayBar
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayValue"             => withNestingArrayValue
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayCtxBar"            => withNestingArrayCtxBar
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayCtxValue?data=100" =>
        withNestingArrayCtxValue(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayCtxValue?data=200" =>
        withNestingArrayCtxValue(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingArrayCtxValue?data=300" =>
        withNestingArrayCtxValue(req)

      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Bar"            => withNestingLevel3Bar
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Baz?data=100"   => withNestingLevel3Baz(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Baz?data=200"   => withNestingLevel3Baz(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Baz?data=300"   => withNestingLevel3Baz(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Value?data=101" => withNestingLevel3Value(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Value?data=201" => withNestingLevel3Value(req)
      case (url, Method.GET) if url == "https://foo.com/withNestingLevel3Value?data=301" => withNestingLevel3Value(req)

      case _ => ZIO.fail(new IllegalArgumentException(s"Invalid request: $req"))
    }
}

object ExecutionSpecHttpClient {
  def default: ZLayer[Any, Throwable, ExecutionSpecHttpClient] = ZLayer.succeed(new ExecutionSpecHttpClient())
}
