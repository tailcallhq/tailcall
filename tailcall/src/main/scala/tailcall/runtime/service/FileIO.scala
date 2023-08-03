package tailcall.runtime.service

import tailcall.runtime.service.FileIO.Flags
import zio.json.{DecoderOps, JsonDecoder}
import zio.{Task, ZIO, ZLayer}

import java.io.File
import java.nio.file.{Files, StandardOpenOption}

trait FileIO {
  def read(file: File): Task[String]
  def write(file: File, content: String, flags: Flags = FileIO.defaultFlag): Task[Unit]
  def readJson[A: JsonDecoder](file: File): Task[A] =
    read(file).flatMap(_.fromJson[A] match {
      case Left(value)  => ZIO.fail(new RuntimeException(value))
      case Right(value) => ZIO.succeed(value)
    })
}

object FileIO {
  def defaultFlag: Flags = Flags(Nil)

  def read(file: File): ZIO[FileIO, Throwable, String] = ZIO.serviceWithZIO(_.read(file))

  def readJson[A: JsonDecoder](file: File): ZIO[FileIO, Throwable, A] = ZIO.serviceWithZIO(_.readJson(file))

  def write(file: File, content: String): ZIO[FileIO, Throwable, Unit] = ZIO.serviceWithZIO(_.write(file, content))

  def default: ZLayer[Any, Nothing, FileIO] =
    ZLayer.succeed(new FileIO {
      override def read(file: File): Task[String] = ZIO.attemptBlocking(Files.readString(file.toPath))
      override def write(file: File, content: String, flags: Flags): Task[Unit] =
        ZIO.attemptBlocking(Files.write(file.toPath, content.getBytes, flags.options: _*)).unit
    })

  final case class Flags(options: List[StandardOpenOption]) {
    def append(option: StandardOpenOption): Flags = copy(options = option :: options)
    def withRead: Flags                           = append(StandardOpenOption.READ)
    def withAppend: Flags                         = append(StandardOpenOption.APPEND)
    def withTruncateExisting: Flags               = append(StandardOpenOption.TRUNCATE_EXISTING)
    def withCreate: Flags                         = append(StandardOpenOption.CREATE)
    def withCreateNew: Flags                      = append(StandardOpenOption.CREATE_NEW)
    def withDeleteOnClose: Flags                  = append(StandardOpenOption.DELETE_ON_CLOSE)
    def withSparse: Flags                         = append(StandardOpenOption.SPARSE)
    def withSync: Flags                           = append(StandardOpenOption.SYNC)
    def withDsync: Flags                          = append(StandardOpenOption.DSYNC)
  }
}
