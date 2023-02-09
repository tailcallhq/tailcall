package tailcall.gateway.internal

import io.netty.bootstrap.Bootstrap
import io.netty.buffer.{ByteBufUtil, Unpooled}
import io.netty.channel._
import io.netty.channel.nio.NioEventLoopGroup
import io.netty.channel.socket.nio.NioSocketChannel
import io.netty.handler.codec.http._
import zio.{UIO, ZIO}

import java.net.URL
import scala.jdk.CollectionConverters.CollectionHasAsScala
import tailcall.gateway.ast.Method

trait HttpClient {
  def request(
    m: Method,
    u: String,
    h: Map[String, String],
    b: Option[String]
  ): HttpClient.AsyncHandler
}

object HttpClient {
  final class NettyHttpClient() extends HttpClient {
    def boostrapConnection(request: HttpRequest)(cb: FullHttpResponse => Any): Bootstrap =
      new Bootstrap().group(new NioEventLoopGroup()).channelFactory(new ChannelFactory[Channel] {
        override def newChannel(): Channel = new NioSocketChannel()
      }).handler(new ChannelInitializer[Channel] {
        override def initChannel(ch: Channel): Unit = {
          ch.pipeline().addLast(new HttpClientCodec()).addLast(new HttpObjectAggregator(1024 * 100))
            .addLast(new SimpleChannelInboundHandler[FullHttpResponse]() {
              override def channelRead0(ctx: ChannelHandlerContext, msg: FullHttpResponse): Unit =
                cb(msg)
              override def channelActive(ctx: ChannelHandlerContext): Unit = ctx
                .writeAndFlush(request)
            })
        }
      })

    override def request(
      m: Method,
      u: String,
      h: Map[String, String],
      b: Option[String]
    ): AsyncHandler = cb => {
      val url  = new URL(u)
      val host = url.getHost
      val port = Math.max(url.getPort, 80)

      val request = buildRequest(m, h, b, url)

      request.headers().set(HttpHeaderNames.HOST, host)

      var close: Option[ChannelFuture] = None

      val future = boostrapConnection(request) { response =>
        val status  = response.status().code()
        val body    = ByteBufUtil.getBytes(response.content)
        val headers = response.headers().entries().asScala
          .foldLeft(Map.empty[String, String])((acc, h) => acc + (h.getKey -> h.getValue))

        close.foreach(_.cancel(true))
        cb(status, headers, body)
      }.connect(host, port)

      close = Some(future)
    }

    private def buildRequest(
      method: Method,
      headers: Map[String, String],
      body: Option[String],
      url: URL
    ): FullHttpRequest = {
      val request = new DefaultFullHttpRequest(
        io.netty.handler.codec.http.HttpVersion.HTTP_1_1,
        io.netty.handler.codec.http.HttpMethod.valueOf(method.name),
        url.getPath,
        body.map(b => io.netty.buffer.Unpooled.wrappedBuffer(b.getBytes))
          .getOrElse(Unpooled.EMPTY_BUFFER)
      )
      request.headers.set(headers.foldLeft[HttpHeaders](new DefaultHttpHeaders())((acc, h) =>
        acc.add(h._1, h._2)
      ))

      request
    }
  }

  type AsyncHandler = ((Int, Map[String, String], Array[Byte]) => Unit) => Any

  def make: UIO[HttpClient] = ZIO.succeed(new NettyHttpClient())
}
