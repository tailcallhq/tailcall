package tailcall.runtime.transcoder

import tailcall.runtime.ast.Endpoint
import tailcall.runtime.dsl.Postman
import tailcall.runtime.http.{HttpClient, Request}
import zio.{Chunk, ZIO}

import java.nio.charset.Charset

trait Postman2Endpoints {
  def toEndpoints(postman: Postman, config: Postman2Endpoints.Config): ZIO[HttpClient, Throwable, List[Endpoint]] = {
    ZIO.foreach(postman.collection.item.filter(item => item.request.url.nonEmpty)) { item =>
      {
        val request = item.request
        for {
          client <- ZIO.service[HttpClient]
          path             = request.url.map(_.path.mkString("/"))
          endpoint         = Endpoint.make(config.host)
          headers          = request.header.map(h => (h.key, h.value)).toMap
          endpointWithPath = if (path.nonEmpty) endpoint.withPath("/" + path.get) else endpoint
          resp    <- client.request(Request(
            config.host + "/" + path.getOrElse(""),
            request.method,
            headers,
            request.body.map(body => Chunk.fromIterable(body.raw.toString().getBytes(Charset.defaultCharset())))
              .getOrElse(Chunk.empty),
          ))
          jsonStr <- resp.body.asString
        } yield endpointWithPath.withMethod(request.method)
          .withInput(request.body.flatMap(body => Transcoder.toTSchema(body.raw.toString()).toOption))
          .withOutput(Transcoder.toTSchema(jsonStr).toOption).withHeader(headers.toList: _*).withAddress(config.host)
      }

    }

  }
}

object Postman2Endpoints {
  final case class Config(allowHttpCalls: Boolean = false, host: String)
}
