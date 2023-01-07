package tailcall.gateway.reader

import tailcall.gateway.adt.Config
import tailcall.gateway.internal.Extension
import zio.{Task, ZIO}

import java.io.File
import java.net.URL
import java.nio.file.Path
import scala.io.Source

/**
 * Reads configuration from a file.
 *
 * TODO: replace the custom implementation with ZIO Config.
 */
trait ConfigReader {
  def readFile(file: => File): Task[Config]
  final def readPath(path: => Path): Task[Config] = readFile(path.toFile)
  final def readURL(url: => URL): Task[Config]    = readFile(new File(url.getPath))
}

object ConfigReader {

  val custom: ConfigReader =
    new ConfigReader {
      override def readFile(file: => File): Task[Config] = {
        for {
          ext    <- Extension.detect(file.getName)
          string <- ZIO.attemptBlocking(Source.fromFile(file).mkString(""))
          config <- ext.decode(string)
        } yield config
      }
    }

  /**
   * Reads configuration from a file using ZIO Config
   */
  def zioConfig: ConfigReader = ???
}
