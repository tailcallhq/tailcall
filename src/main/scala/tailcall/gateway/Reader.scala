package tailcall.gateway

import caliban.parsing.Parser
import caliban.parsing.adt.Document
import tailcall.gateway.adt.Config
import tailcall.gateway.internal.Extension
import zio.{Task, ZIO}

import java.io.File
import java.net.URL
import java.nio.file.Path
import scala.io.Source

/**
 * Reads configuration from a file.
 */
trait Reader[A] {
  def readFile(file: => File): Task[A]
  final def readPath(path: => Path): Task[A] = readFile(path.toFile)
  final def readURL(url: => URL): Task[A]    = readFile(new File(url.getPath))
}

object Reader {

  // TODO: replace the custom implementation with ZIO Config.
  def config: Reader[Config] =
    new Reader[Config] {
      override def readFile(file: => File): Task[Config] = {
        for {
          ext    <- Extension.detect(file.getName)
          string <- ZIO.attemptBlocking(Source.fromFile(file).mkString(""))
          config <- ext.decode(string)
        } yield config
      }
    }

  def document: Reader[Document] =
    new Reader[Document] {
      override def readFile(file: => File): Task[Document] = {
        for {
          string   <- ZIO.attemptBlocking(Source.fromFile(file).mkString(""))
          document <- Parser.parseQuery(string).mapError(_ => ???)
        } yield document
      }
    }
}
