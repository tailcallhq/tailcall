package tailcall.runtime.model

import tailcall.runtime.http.{Method, Request, Scheme}
import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.model.Mustache.MustacheExpression
import tailcall.runtime.transcoder.Transcoder
import zio.Chunk
import zio.schema.{DynamicValue, Schema}

final case class Endpoint(
  method: Method = Method.GET,
  path: Path = Path.empty,
  query: Chunk[(String, String)] = Chunk.empty,
  address: Endpoint.InetAddress,
  input: Option[TSchema] = None,
  output: Option[TSchema] = None,
  headers: Chunk[(String, String)] = Chunk.empty,
  scheme: Scheme = Scheme.Http,
  body: Option[MustacheExpression] = None,
  description: Option[String] = None,
) {
  self =>
  lazy val outputSchema: Schema[Any] = output.map(TSchema.toZIOSchema).getOrElse(Schema[Unit]).asInstanceOf[Schema[Any]]
  lazy val inputSchema: Schema[Any]  = input.map(TSchema.toZIOSchema).getOrElse(Schema[Unit]).asInstanceOf[Schema[Any]]

  def evaluate(input: DynamicValue): Request = Endpoint.evaluate(self, input)

  // TODO: add unit tests
  def url: String = {
    val portString = address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString        = query.nonEmptyOrElse("")(_.map { case (k, v) => s"$k=$v" }.mkString("?", "&", ""))
    val pathString: String = path.encode.getOrElse(throw new RuntimeException("Path encoding failed"))
    List(scheme.name, "://", address.host, portString, pathString, queryString).mkString
  }

  def withAddress(address: Endpoint.InetAddress): Endpoint = copy(address = address)

  def withAddress(address: String): Endpoint = copy(address = Endpoint.inet(address))

  def withBody(body: MustacheExpression): Endpoint = copy(body = Option(body))

  def withBody(body: String): Endpoint = copy(body = MustacheExpression.syntax.parseString(body).toOption)

  def withDescription(description: String): Endpoint = copy(description = Option(description))

  def withHeader(headers: (String, String)*): Endpoint = copy(headers = Chunk.from(headers))

  def withHeaders(headers: Map[String, String]): Endpoint = copy(headers = Chunk.from(headers))

  def withHttp: Endpoint = withScheme(Scheme.Http)

  def withHttps: Endpoint = withScheme(Scheme.Https)

  def withInput(schema: Option[TSchema]): Endpoint = copy(input = schema)

  def withMethod(method: Method): Endpoint = copy(method = method)

  def withOutput(schema: Option[TSchema]): Endpoint = copy(output = schema)

  def withPath(path: Path): Endpoint = copy(path = path)

  def withPath(path: String): Endpoint = copy(path = Path.unsafe.fromString(path))

  def withPort(port: Int): Endpoint = {
    if (port < 0 || port > 65535) throw new IllegalArgumentException("Port must be between 0 and 65535")
    copy(address = address.copy(port = port))
  }

  def withQuery(query: (String, String)*): Endpoint = copy(query = Chunk.from(query))

  def withQuery(query: Map[String, String]): Endpoint = copy(query = Chunk.from(query))

  def withScheme(scheme: Scheme): Endpoint = copy(scheme = scheme)
}

object Endpoint {
  def evaluate(endpoint: Endpoint, input: DynamicValue): Request = {
    val method     = endpoint.method
    val portString = endpoint.address.port match {
      case 80   => ""
      case 443  => ""
      case port => s":$port"
    }

    val queryString = endpoint.query.nonEmptyOrElse("")(_.map { case (key, string) =>
      MustacheExpression.syntax.parseString(string) match {
        case Left(_)           => Some(s"$key=${string}")
        case Right(expression) => DynamicValueUtil.getPath(input, expression.path.toList, true) match {
            case Some(DynamicValue.Sequence(values)) =>
              Some(values.toSet.flatMap(DynamicValueUtil.asString).map(input => s"$key=${input}").mkString("&"))
            case _ => MustacheExpression.evaluate(string, input).map(str => s"$key=$str").toOption
          }
      }
    }.collect { case Some(a) => a }.mkString("?", "&", ""))

    val pathString: String = endpoint.path.unsafeEval(input)

    val url = List(endpoint.scheme.name, "://", endpoint.address.host, portString, pathString, queryString).mkString

    val headers = endpoint.headers.map { case (k, v) =>
      MustacheExpression.syntax.parseString(v) match {
        case Left(_)  => Some((k, v))
        case Right(v) => MustacheExpression.evaluate(v, input).map((k, _)).toOption
      }
    }.collect { case Some(kv) => kv }.toMap

    val bodyDynamic = endpoint.body match {
      case Some(value) => DynamicValueUtil.getPath(input, value.path.toList)
      case None        => Some(input)
    }

    val body =
      if (method == Method.GET || method == Method.DELETE) Chunk.empty
      else for {
        dynamic <- Chunk.fromIterable(bodyDynamic)
        json    <- Chunk.fromIterable(Transcoder.toJson(dynamic).toOption)
        chunk   <- Chunk.fromArray(json.toJson.getBytes())
      } yield chunk

    val request = Request(
      method = method,
      url = url,
      headers = headers ++ Map("content-length" -> body.size.toString, "content-type" -> "application/json"),
    )
    if (body.nonEmpty && method != Method.GET) request.withBody(body) else request
  }

  def from(url: String): Endpoint = {
    val uri     = new java.net.URI(url)
    val path    = Path.unsafe.fromString(uri.getPath())
    val query   = Option(uri.getQuery).fold(Chunk.empty[(String, String)]) { query =>
      Chunk.from(query.split("&").map(_.split("=")).map { case Array(k, v) => k -> v })
    }
    val address = InetAddress(uri.getHost, uri.getPort)
    Endpoint(path = path, query = query, address = address)
  }

  def get(address: String): Endpoint = make(address).withMethod(Method.GET)

  def inet(host: String, port: Int = 80): InetAddress = InetAddress(host, port)

  def make(address: String): Endpoint = Endpoint(address = Endpoint.inet(address))

  def post(address: String): Endpoint = make(address).withMethod(Method.POST)

  final case class InetAddress(host: String, port: Int = 80)
}
