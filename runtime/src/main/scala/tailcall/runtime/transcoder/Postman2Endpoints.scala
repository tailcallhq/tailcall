package tailcall.runtime.transcoder

import tailcall.runtime.ast.Endpoint
import tailcall.runtime.dsl.Postman
import tailcall.runtime.http.{HttpClient, Request}
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import zio.{Chunk, ZIO}

import java.nio.charset.Charset

trait Postman2Endpoints {
  def toEndpoints(postman: Postman): ZIO[HttpClient, Throwable, List[Endpoint]] = {
    ZIO.foreach(postman.collection.item.filter(item => item.request.url.nonEmpty)) { item =>
      {
        val request = item.request

        for {
          client <- ZIO.service[HttpClient]
          host             = request.url match {
            case Some(value) => value.protocol.name + "://" + value.host.mkString(".")
            case None        => throw new Exception("Host is not defined")
          }
          path             = request.url.map(_.path.mkString("/"))
          _                = Console.println(s"making call to host: $host, path: $path")
          endpoint         = Endpoint.make(host).withDescription(item.name)
          headers          = request.header.map(h => (h.key, h.value)).toMap
          endpointWithPath = if (path.nonEmpty) endpoint.withPath("/" + path.get) else endpoint
          endpointWithHost = endpointWithPath.withAddress(host)
          resp    <- client.request(Request(
            host + "/" + path.getOrElse(""),
            request.method,
            headers,
            request.body.map(body => Chunk.fromIterable(body.raw.toString().getBytes(Charset.defaultCharset())))
              .getOrElse(Chunk.empty),
          ))
          jsonStr <- resp.body.asString
        } yield endpointWithHost.withMethod(request.method)
          .withInput(request.body.flatMap(body => Transcoder.toTSchema(body.raw.toString()).toOption))
          .withOutput(Transcoder.toTSchema(jsonStr).toOption).withHeader(headers.toList: _*)
      }

    }

  }
}

object Postman2Endpoints {
  final case class Config(allowHttpCalls: Boolean = false, nameGen: NameGenerator)
}
