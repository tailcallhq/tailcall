package tailcall.runtime

import tailcall.runtime.openApi.YamlParser
import tailcall.runtime.service.{FileIO, OpenAPI2Config}
import zio.ZIO
import zio.test.Assertion.anything
import zio.test.{ZIOSpecDefault, _}

import java.io.File

object OpenApiSpec extends ZIOSpecDefault {
  def spec =
    suite("OpenApiSpec")(test("parseyaml") {
      val program = for {
        fileIO  <- ZIO.service[FileIO]
        yaml    <- fileIO.read(new File(getClass.getResource("OpenAPI.yml").getPath))
        encoded <- ZIO.fromEither(YamlParser.parseFile(yaml))
        _ = OpenAPI2Config.convert(encoded)
      } yield ()
      assertZIO(program)(anything)
    }).provide(FileIO.default)

}
