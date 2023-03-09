package tailcall.runtime.http

import io.netty.bootstrap.Bootstrap
import io.netty.buffer.ByteBufUtil
import io.netty.channel._
import io.netty.channel.nio.NioEventLoopGroup
import io.netty.channel.socket.nio.NioSocketChannel
import io.netty.handler.codec.http._
import zio.{ZIO, ZLayer}

import java.net.URL
import scala.jdk.CollectionConverters.CollectionHasAsScala

trait HttpClient {
  def request(req: HttpRequest): HttpClient.AsyncHandler
}

// TODO: handle cancellation
object HttpClient {
  final class Live(bootstrap: Bootstrap) extends HttpClient {
    def bootstrapConnection(request: HttpRequest)(cb: Response => Any): Bootstrap =
      bootstrap.handler(new ChannelInitializer[Channel] {
        override def initChannel(ch: Channel): Unit = {
          ch.pipeline().addLast(new HttpClientCodec()).addLast(new HttpObjectAggregator(1024 * 1000))
            .addLast(new SimpleChannelInboundHandler[FullHttpResponse](false) {
              override def channelRead0(ctx: ChannelHandlerContext, msg: FullHttpResponse): Unit = {
                val code    = msg.status().code
                val headers = msg.headers().entries().asScala
                  .foldLeft(Map.empty[String, String])((acc, h) => acc + (h.getKey -> h.getValue))
                val bytes   = ByteBufUtil.getBytes(msg.content())
                msg.content().release(msg.content().refCnt())
                cb((code, headers, bytes))
                ctx.close()
              }

              override def channelActive(ctx: ChannelHandlerContext): Unit = ctx.writeAndFlush(request)
            })
        }
      })

    override def request(request: HttpRequest): AsyncHandler = { cb =>
      val url  = new URL(request.uri())
      val host = url.getHost
      val port = Math.max(url.getPort, 80)

      var close: Option[ChannelFuture] = None

      request.headers().set(HttpHeaderNames.HOST, host)
      request.headers().set(HttpHeaderNames.USER_AGENT, "tailcall-gateway/netty")
      request.headers().set(HttpHeaderNames.ACCEPT, "*/*")
      request.headers().set(HttpHeaderNames.CONNECTION, "close")
      val future = bootstrapConnection(request) { case (status, headers, body) =>
        close.foreach(_.cancel(true))
        cb(status, headers, body)
      }.connect(host, port)

      close = Some(future)
      () => future.cancel(true)
    }

  }

  type Close        = () => Unit
  type Response     = (Int, Map[String, String], Array[Byte])
  type AsyncHandler = (Response => Unit) => Close

  def live: ZLayer[Any, Nothing, HttpClient] = {
    ZLayer.scoped {
      for {
        group <- ZIO.succeed(new NioEventLoopGroup())
        _     <- ZIO.addFinalizer(ZIO.succeed(group.shutdownGracefully()))
      } yield new Live(new Bootstrap().group(group).channelFactory {
        new ChannelFactory[Channel] {
          override def newChannel(): Channel = new NioSocketChannel()
        }
      })
    }
  }
}
