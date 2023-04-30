package tailcall.runtime.service

import tailcall.runtime.http.Request
import tailcall.runtime.model.{Endpoint, Postman}
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import tailcall.runtime.transcoder.Transcoder
import zio.{Chunk, Task, ZIO, ZLayer}

import java.nio.charset.{Charset, StandardCharsets}

trait EndpointGenerator {
  def generate(postman: Postman): Task[List[Endpoint]]
}

object EndpointGenerator {
  def default: ZLayer[Any, Throwable, EndpointGenerator] =
    HttpCache.default >>> DataLoader.http >>> HttpContext.default >>> live

  def live: ZLayer[HttpContext, Nothing, EndpointGenerator] = ZLayer.fromFunction(dataLoader => Live(dataLoader))

  def generate(postman: Postman): ZIO[EndpointGenerator, Throwable, List[Endpoint]] =
    ZIO.serviceWithZIO(_.generate(postman))

  final case class Live(httpContext: HttpContext) extends EndpointGenerator {
    override def generate(postman: Postman): Task[List[Endpoint]] = {
      ZIO.foreach(postman.collection.item.filter(item => item.request.url.nonEmpty)) { item =>
        {
          val request          = item.request
          val host             = request.url match {
            case Some(value) => value.protocol.name + "://" + value.host.mkString(".")
            case None        => throw new Exception("Host is not defined")
          }
          val path             = request.url.map(_.path.mkString("/"))
          val endpoint         = Endpoint.make(host).withDescription(item.name)
          val headers          = request.header.map(h => (h.key, h.value)).toMap
          val endpointWithPath = if (path.nonEmpty) endpoint.withPath("/" + path.get) else endpoint
          val endpointWithHost = endpointWithPath.withAddress(host)
          httpContext.dataLoader.load(Request(
            host + "/" + path.getOrElse(""),
            request.method,
            headers,
            request.body.map(body => Chunk.fromIterable(body.raw.toString().getBytes(Charset.defaultCharset())))
              .getOrElse(Chunk.empty),
          )).map(resp =>
            endpointWithHost.withMethod(request.method)
              .withInput(request.body.flatMap(body => Transcoder.toTSchema(body.raw.toString()).toOption))
              .withOutput(Transcoder.toTSchema(new String(resp.toArray, StandardCharsets.UTF_8)).toOption)
              .withHeader(headers.toList: _*)
          )
        }
      }
    }

  }

  final case class Config(allowHttpCalls: Boolean = false, nameGen: NameGenerator)
}
