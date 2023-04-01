package tailcall.runtime.transcoder

import tailcall.runtime.ast.Endpoint
import tailcall.runtime.dsl.Postman
import tailcall.runtime.http.Request
import tailcall.runtime.service.DataLoader.HttpDataLoader
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import zio.{Chunk, ZIO}

import java.nio.charset.{Charset, StandardCharsets}

trait Postman2Endpoints {
  def toEndpoints(postman: Postman): ZIO[HttpDataLoader, Throwable, List[Endpoint]] = {
    ZIO.foreach(postman.collection.item.filter(item => item.request.url.nonEmpty)) { item =>
      {
        val request = item.request

        for {
          dataloader <- ZIO.service[HttpDataLoader]
          host             = request.url match {
            case Some(value) => value.protocol.name + "://" + value.host.mkString(".")
            case None        => throw new Exception("Host is not defined")
          }
          path             = request.url.map(_.path.mkString("/"))
          endpoint         = Endpoint.make(host).withDescription(item.name)
          headers          = request.header.map(h => (h.key, h.value)).toMap
          endpointWithPath = if (path.nonEmpty) endpoint.withPath("/" + path.get) else endpoint
          endpointWithHost = endpointWithPath.withAddress(host)
          resp <- dataloader.load(Request(
            host + "/" + path.getOrElse(""),
            request.method,
            headers,
            request.body.map(body => Chunk.fromIterable(body.raw.toString().getBytes(Charset.defaultCharset())))
              .getOrElse(Chunk.empty),
          ))
        } yield endpointWithHost.withMethod(request.method)
          .withInput(request.body.flatMap(body => Transcoder.toTSchema(body.raw.toString()).toOption))
          .withOutput(Transcoder.toTSchema(new String(resp.toArray, StandardCharsets.UTF_8)).toOption)
          .withHeader(headers.toList: _*)
      }

    }

  }
}

object Postman2Endpoints {
  final case class Config(allowHttpCalls: Boolean = false, nameGen: NameGenerator)
}
